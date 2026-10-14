use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};

const RUST_TOOLBAR: &str = "<div class=\"code-block\"><div class=\"code-block-toolbar\"><span class=\"code-language\">Rust</span></div>";
const SHELL_TOOLBAR: &str = "<div class=\"code-block\"><div class=\"code-block-toolbar\"><span class=\"code-language\">Shell</span></div>";
const TEXT_TOOLBAR: &str = "<div class=\"code-block\"><div class=\"code-block-toolbar\"><span class=\"code-language\">Text</span></div>";

fn select_language_label<'a>(kind: &'a CodeBlockKind<'_>) -> &'a str {
    let CodeBlockKind::Fenced(info) = kind else {
        return "Text";
    };
    let token = info.split_whitespace().next().unwrap_or_default();
    if token.eq_ignore_ascii_case("rust") || token.eq_ignore_ascii_case("rs") {
        "Rust"
    } else if token.eq_ignore_ascii_case("sh") || token.eq_ignore_ascii_case("shell") {
        "Shell"
    } else if token.is_empty()
        || token.eq_ignore_ascii_case("text")
        || token.eq_ignore_ascii_case("plaintext")
        || token.eq_ignore_ascii_case("txt")
    {
        "Text"
    } else {
        token
    }
}

fn render_opening_toolbar(kind: &CodeBlockKind<'_>) -> Event<'static> {
    let label = select_language_label(kind);
    match label {
        "Rust" => return Event::Html(CowStr::Borrowed(RUST_TOOLBAR)),
        "Shell" => return Event::Html(CowStr::Borrowed(SHELL_TOOLBAR)),
        "Text" => return Event::Html(CowStr::Borrowed(TEXT_TOOLBAR)),
        _ => {}
    }
    let mut markup = String::from(
        "<div class=\"code-block\"><div class=\"code-block-toolbar\"><span class=\"code-language\">",
    );
    for character in label.chars() {
        match character {
            '&' => markup.push_str("&amp;"),
            '<' => markup.push_str("&lt;"),
            '>' => markup.push_str("&gt;"),
            '"' => markup.push_str("&quot;"),
            '\'' => markup.push_str("&#39;"),
            _ => markup.push(character),
        }
    }
    markup.push_str("</span></div>");
    Event::Html(CowStr::Boxed(markup.into_boxed_str()))
}

pub(super) fn decorate_code_blocks<'a>(
    mut events: impl Iterator<Item = Event<'a>>,
) -> impl Iterator<Item = Event<'a>> {
    let mut pending = None;
    std::iter::from_fn(move || {
        if let Some(event) = pending.take() {
            return Some(event);
        }
        let event = events.next()?;
        if let Event::Start(Tag::CodeBlock(kind)) = &event {
            let toolbar = render_opening_toolbar(kind);
            pending = Some(event);
            return Some(toolbar);
        }
        if matches!(event, Event::End(TagEnd::CodeBlock)) {
            pending = Some(Event::Html(CowStr::Borrowed("</div>\n")));
        }
        Some(event)
    })
}
