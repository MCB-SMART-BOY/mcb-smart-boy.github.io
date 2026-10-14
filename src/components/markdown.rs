use leptos::{html, prelude::*};

#[cfg(feature = "hydrate")]
mod code_copy;

/// Keeps the rendered article unchanged; copy controls mount only after hydration.
#[component]
pub(crate) fn MarkdownContent(html: &'static str, class: &'static str) -> impl IntoView {
    let root = NodeRef::<html::Div>::new();
    #[cfg(feature = "hydrate")]
    code_copy::install_copy_controls(root);
    view! { <div node_ref=root class=class lang="zh-CN" inner_html=html></div> }
}
