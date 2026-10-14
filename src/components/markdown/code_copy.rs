use crate::{
    clipboard::{ClipboardError, classify_js_error, write_clipboard_text},
    components::icons::{Icon, IconKind},
    locale::Locale,
};
use leptos::{html, portal::Portal, prelude::*};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use wasm_bindgen::{JsCast, JsValue};

#[derive(Clone, Copy, PartialEq, Eq)]
enum CopyState {
    Idle,
    Copying,
    Copied,
    Failed,
}

struct CodeTarget {
    code: web_sys::Element,
    language: String,
    index: u32,
}

/// A broken generated DOM contract is fatal to enhancement, not to readable code.
enum CodeControlsError {
    /// A required, renderer-owned node is missing or has the wrong type.
    MissingNode(&'static str),
    /// A DOM query failed; keep the browser exception without logging its payload.
    Query {
        selector: &'static str,
        source: JsValue,
    },
}

fn find_element(
    root: &web_sys::Element,
    selector: &'static str,
) -> Result<web_sys::Element, CodeControlsError> {
    root.query_selector(selector)
        .map_err(|source| CodeControlsError::Query { selector, source })?
        .ok_or(CodeControlsError::MissingNode(selector))
}

fn mount_copy_control(
    block: web_sys::Element,
    index: u32,
    locale: RwSignal<Locale>,
) -> Result<(), CodeControlsError> {
    let toolbar = find_element(&block, ":scope > .code-block-toolbar")?;
    let code = find_element(&block, ":scope > pre > code")?;
    let language = find_element(&toolbar, ":scope > .code-language")?
        .text_content()
        .unwrap_or_default();
    let target = StoredValue::new_local(CodeTarget {
        code,
        language,
        index,
    });
    // Portal owns its mounted view and removes it with this Markdown owner.
    let _ = view! { <Portal mount=toolbar><CodeCopyButton target locale/></Portal> };
    Ok(())
}

fn mount_copy_controls(
    root: &web_sys::Element,
    locale: RwSignal<Locale>,
) -> Result<(), CodeControlsError> {
    let selector = ".code-block";
    let blocks = root
        .query_selector_all(selector)
        .map_err(|source| CodeControlsError::Query { selector, source })?;
    for index in 0..blocks.length() {
        let block = blocks
            .item(index)
            .and_then(|node| node.dyn_into::<web_sys::Element>().ok())
            .ok_or(CodeControlsError::MissingNode(selector))?;
        mount_copy_control(block, index + 1, locale)?;
    }
    Ok(())
}

pub(super) fn install_copy_controls(root: NodeRef<html::Div>) {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    Effect::new(move |_| {
        let Some(root) = root.get() else { return };
        if let Err(error) = mount_copy_controls(&root, locale) {
            match error {
                CodeControlsError::MissingNode(selector) => leptos::logging::error!(
                    "Failed to enhance Markdown code blocks: missing {selector}"
                ),
                CodeControlsError::Query { selector, source } => leptos::logging::error!(
                    "Failed to enhance Markdown code blocks: query {selector} ({})",
                    classify_js_error(&source)
                ),
            }
        }
    });
}

fn finish_copy(
    target: StoredValue<CodeTarget, LocalStorage>,
    state: RwSignal<CopyState>,
    is_alive: &AtomicBool,
    result: Result<(), ClipboardError>,
) {
    let outcome = match result {
        Ok(()) => CopyState::Copied,
        Err(error) => {
            // Permissions/capability failures are user-fixable: keep manual selection available.
            leptos::logging::error!(
                "Code block copy failed: {} ({})",
                error.category(),
                error.reason_kind()
            );
            CopyState::Failed
        }
    };
    // A pending Clipboard request must never write into a later article's controls.
    if is_alive.load(Ordering::Relaxed) && target.with_value(|target| target.code.is_connected()) {
        state.set(outcome);
    }
}

fn start_copy(
    target: StoredValue<CodeTarget, LocalStorage>,
    state: RwSignal<CopyState>,
    is_alive: Arc<AtomicBool>,
) {
    if state.get_untracked() == CopyState::Copying {
        return;
    }
    state.set(CopyState::Copying);
    let text = target.with_value(|target| target.code.text_content().unwrap_or_default());
    // Invoke in the click handler, before any await, to retain browser user activation.
    let pending = match write_clipboard_text(&text) {
        Ok(pending) => pending,
        Err(error) => {
            finish_copy(target, state, &is_alive, Err(error));
            return;
        }
    };
    leptos::task::spawn_local(async move {
        let result = wasm_bindgen_futures::JsFuture::from(pending)
            .await
            .map(|_| ())
            .map_err(ClipboardError::Rejected);
        finish_copy(target, state, &is_alive, result);
    });
}

fn select_button_label(state: CopyState, locale: Locale) -> &'static str {
    match state {
        CopyState::Idle => locale.select("复制", "Copy"),
        CopyState::Copying => locale.select("复制中…", "Copying…"),
        CopyState::Copied => locale.select("已复制", "Copied"),
        CopyState::Failed => locale.select("重试复制", "Retry copy"),
    }
}

fn select_copy_feedback(state: CopyState, locale: Locale) -> &'static str {
    match state {
        CopyState::Idle => "",
        CopyState::Copying => locale.select("正在复制代码…", "Copying code…"),
        CopyState::Copied => locale.select("已复制", "Copied"),
        CopyState::Failed => locale.select(
            "复制失败，请手动选择代码复制。",
            "Copy failed. Select and copy the code manually.",
        ),
    }
}

#[component]
fn CodeCopyButton(
    target: StoredValue<CodeTarget, LocalStorage>,
    locale: RwSignal<Locale>,
) -> impl IntoView {
    let state = RwSignal::new(CopyState::Idle);
    // Owner-local liveness also guards completions after the DOM has been removed.
    let is_alive = Arc::new(AtomicBool::new(true));
    let cleanup_alive = Arc::clone(&is_alive);
    on_cleanup(move || cleanup_alive.store(false, Ordering::Relaxed));
    view! {
        <button type="button" class="code-copy-button"
            lang=move || locale.get().as_str()
            title=move || select_button_label(state.get(), locale.get())
            aria-label=move || target.with_value(|target| format!("{} {} ({})",
                locale.get().select("复制代码：", "Copy code:"), target.language, target.index))
            disabled=move || state.get() == CopyState::Copying
            on:click=move |_| start_copy(target, state, Arc::clone(&is_alive))>
            {move || view! { <Icon kind=if state.get() == CopyState::Copied {
                IconKind::Check
            } else {
                IconKind::Copy
            }/> }}
        </button>
        <span class="sr-only" role="status" aria-live="polite" aria-atomic="true"
            lang=move || locale.get().as_str()>
            {move || select_copy_feedback(state.get(), locale.get())}
        </span>
        <p class="code-copy-status" class:is-error=move || state.get() == CopyState::Failed aria-hidden="true"
            lang=move || locale.get().as_str() hidden=move || state.get() == CopyState::Idle>
            {move || select_copy_feedback(state.get(), locale.get())}
        </p>
    }
}
