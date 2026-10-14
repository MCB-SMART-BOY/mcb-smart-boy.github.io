use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, TagEnd, html};

#[path = "markdown/code_blocks.rs"]
mod code_blocks;

#[path = "../build_support/links.rs"]
mod links;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HeadingEntry {
    pub(crate) id: String,
    pub(crate) text: String,
    pub(crate) level: u8,
}

pub(crate) struct RenderedMarkdown {
    pub(crate) html: String,
    pub(crate) headings: Vec<HeadingEntry>,
}
const FIRST_SECTION_NUMBER: usize = 1;

fn is_safe_destination(destination: &str) -> bool {
    destination.starts_with("https://")
        || destination.starts_with("http://")
        || destination.starts_with("mailto:")
        || (destination.starts_with('/') && !destination.starts_with("//"))
        || destination.starts_with('#')
}

fn safe_destination(destination: CowStr<'_>) -> CowStr<'_> {
    if is_safe_destination(&destination) {
        destination
    } else {
        CowStr::Borrowed("#")
    }
}

fn safe_link_destination<'a>(destination: CowStr<'a>, source_file: &str) -> CowStr<'a> {
    match links::resolve_content_link(source_file, &destination) {
        Ok(Some(path)) => CowStr::Boxed(path.into_boxed_str()),
        Ok(None) if is_safe_destination(&destination) => destination,
        Ok(None) => CowStr::Borrowed("#"),
        Err(source) => {
            let message = format!("Invalid Markdown link {destination} in {source_file}: {source}");
            #[cfg(feature = "hydrate")]
            web_sys::console::error_1(&message.into());
            #[cfg(not(feature = "hydrate"))]
            eprintln!("{message}");
            CowStr::Borrowed("#")
        }
    }
}

fn sanitize_event<'a>(event: Event<'a>, source_file: &str) -> Event<'a> {
    match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: safe_link_destination(dest_url, source_file),
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Image {
            link_type,
            dest_url: safe_destination(dest_url),
            title,
            id,
        }),
        other => other,
    }
}

pub(crate) fn render_content_markdown(source: &str, source_file: &str) -> RenderedMarkdown {
    let mut headings = Vec::new();
    let mut current_heading: Option<HeadingEntry> = None;
    let mut next_section = FIRST_SECTION_NUMBER;
    let events = Parser::new_ext(source, Options::ENABLE_STRIKETHROUGH).map(|event| {
        let event = sanitize_event(event, source_file);
        let event = match event {
            Event::Start(Tag::Heading {
                level,
                classes,
                attrs,
                ..
            }) => {
                let id = format!("section-{next_section}");
                next_section += 1;
                current_heading = Some(HeadingEntry {
                    id: id.clone(),
                    text: String::new(),
                    level: level as u8,
                });
                Event::Start(Tag::Heading {
                    level,
                    id: Some(CowStr::Boxed(id.into_boxed_str())),
                    classes,
                    attrs,
                })
            }
            other => other,
        };
        match &event {
            Event::Text(text) | Event::Code(text) => {
                if let Some(heading) = current_heading.as_mut() {
                    heading.text.push_str(text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(heading) = current_heading.as_mut() {
                    heading.text.push(' ');
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(mut heading) = current_heading.take() {
                    heading.text = normalize_heading_text(&heading.text);
                    if !heading.text.is_empty() {
                        headings.push(heading);
                    }
                }
            }
            _ => {}
        }
        event
    });
    let events = code_blocks::decorate_code_blocks(events);
    let mut html = String::new();
    html::push_html(&mut html, events);
    RenderedMarkdown { html, headings }
}

#[cfg(test)]
fn render_markdown(source: &str) -> RenderedMarkdown {
    render_content_markdown(source, "/writing/_index.md")
}

fn normalize_heading_text(text: &str) -> String {
    let mut words = text.split_whitespace();
    let mut normalized = words.next().unwrap_or_default().to_owned();
    for word in words {
        normalized.push(' ');
        normalized.push_str(word);
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::{is_safe_destination, render_content_markdown, render_markdown};

    #[test]
    fn markdown_raw_html_and_unsafe_link_are_not_executable() {
        let html =
            render_markdown("Hello <script>alert(1)</script> [click](javascript:alert)").html;
        assert!(!html.contains("<script>"));
        assert!(!html.contains("href=\"javascript:"));
        assert!(html.contains("href=\"#\""));
    }

    #[test]
    fn markdown_https_link_and_code_remain_readable() {
        let html =
            render_markdown("[Rust](https://example.org/README.md?raw=true)\n\n`let x = 1;`").html;
        assert!(html.contains("href=\"https://example.org/README.md?raw=true\""));
        assert!(html.contains("<code>let x = 1;</code>"));
        assert!(!is_safe_destination("//example.com"));
    }

    #[test]
    fn markdown_fenced_code_preserves_whitespace_unicode_and_html_as_text() {
        let html = render_markdown(
            "```rs title=\"ignored\"\n  雪 α\n\n    & <script>alert(1)</script>\n```\n",
        )
        .html;
        assert!(html.contains(
            "<code class=\"language-rs\">  雪 α\n\n    &amp; &lt;script&gt;alert(1)&lt;/script&gt;\n</code>"
        ));
        assert!(html.contains("<span class=\"code-language\">Rust</span>"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn markdown_unmarked_and_empty_code_blocks_display_text() {
        let html = render_markdown("```\n```\n\n```plaintext\n\n```\n\n    indented\n").html;
        assert_eq!(
            html.matches("<span class=\"code-language\">Text</span>")
                .count(),
            3
        );
        assert!(html.contains("<pre><code></code></pre>"));
        assert!(html.contains("<pre><code class=\"language-plaintext\">\n</code></pre>"));
        assert!(html.contains("<pre><code>indented\n</code></pre>"));
    }

    #[test]
    fn markdown_code_blocks_escape_unknown_info_and_keep_nested_blocks_separate() {
        let rendered = render_markdown(
            "# Start\n\n> ~~~<svg/onload=\"x\"&'y> extra\n> # not a heading\n> ~~~\n\n- item\n\n  ```shell\n  echo hello\n  ```\n",
        );
        assert_eq!(rendered.headings.len(), 1);
        assert_eq!(rendered.headings[0].text, "Start");
        assert!(rendered.html.contains(
            "<span class=\"code-language\">&lt;svg/onload=&quot;x&quot;&amp;&#39;y&gt;</span>"
        ));
        assert!(!rendered.html.contains("<svg"));
        assert!(
            rendered
                .html
                .contains("<span class=\"code-language\">Shell</span>")
        );
        assert_eq!(
            rendered.html.matches("<div class=\"code-block\">").count(),
            2
        );
        assert_eq!(rendered.html.matches("</code></pre>\n</div>").count(), 2);
        assert!(
            rendered
                .html
                .contains("<blockquote>\n<div class=\"code-block\">")
        );
        assert!(rendered.html.contains("</div>\n</li>"));
    }

    #[test]
    fn markdown_headings_have_matching_unique_ids_and_preserve_structure() {
        let rendered = render_markdown(
            "# 重复\n\n## 重复\n\n#### [链接](https://example.com) `code`\n\n```md\n# not a heading\n```\n",
        );

        assert_eq!(
            rendered
                .headings
                .iter()
                .map(|heading| (heading.id.as_str(), heading.level, heading.text.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("section-1", 1, "重复"),
                ("section-2", 2, "重复"),
                ("section-3", 4, "链接 code"),
            ]
        );
        assert!(rendered.html.contains("<h1 id=\"section-1\">"));
        assert!(rendered.html.contains("<h2 id=\"section-2\">"));
        assert!(rendered.html.contains("<h4 id=\"section-3\">"));
        assert!(rendered.html.contains("href=\"https://example.com\""));
        assert!(
            !rendered
                .headings
                .iter()
                .any(|heading| heading.text == "not a heading")
        );
    }

    #[test]
    fn markdown_empty_heading_has_an_id_but_no_outline_entry() {
        let rendered = render_markdown("#\n\nParagraph");

        assert!(rendered.headings.is_empty());
        assert!(rendered.html.contains("<h1 id=\"section-1\"></h1>"));
    }
    #[test]
    fn markdown_relative_content_links_use_canonical_paths_and_keep_heading_anchors() {
        let rendered = render_content_markdown(
            "## Intro\n\n[previous](../intro.md#section-1) [parent](_index.md)",
            "/writing/book/chapter/story.md",
        );
        assert!(
            rendered
                .html
                .contains("href=\"/writing/book/intro/#section-1\"")
        );
        assert!(rendered.html.contains("href=\"/writing/book/chapter/\""));
        assert_eq!(rendered.headings[0].id, "section-1");
    }

    #[test]
    fn markdown_index_links_resolve_root_and_parent_directories() {
        let root = render_content_markdown(
            "[self](_index.md) [child](book/deep/_index.md)",
            "/writing/_index.md",
        );
        assert!(root.html.contains("href=\"/writing/\""));
        assert!(root.html.contains("href=\"/writing/book/deep/\""));

        let nested = render_content_markdown(
            "[parent](../_index.md) [root](../../_index.md)",
            "/writing/book/deep/_index.md",
        );
        assert!(nested.html.contains("href=\"/writing/book/\""));
        assert!(nested.html.contains("href=\"/writing/\""));
    }
}
