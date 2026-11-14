use leptos::{html, prelude::*};

#[cfg(feature = "hydrate")]
use super::share_model::ManualPanel;
#[cfg(feature = "hydrate")]
use super::share_model::{PendingShare, ShareError, ShareOperation, is_current_request, route_key};
use super::{
    icons::{Icon, IconKind},
    share_model::{ManualReason, ShareFeedback, ShareState},
};
#[cfg(feature = "hydrate")]
use crate::clipboard::{ClipboardError, write_clipboard_text};
use crate::locale::Locale;
#[cfg(feature = "hydrate")]
use leptos_router::{hooks::use_location, location::Location};
#[cfg(feature = "hydrate")]
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, JsValue};

#[cfg(feature = "hydrate")]
fn find_method(object: &JsValue, name: &str) -> Result<Option<js_sys::Function>, ShareError> {
    let value =
        js_sys::Reflect::get(object, &JsValue::from_str(name)).map_err(ShareError::Invocation)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    value
        .dyn_into::<js_sys::Function>()
        .map(Some)
        .map_err(|_| ShareError::InvalidMethod)
}

#[cfg(feature = "hydrate")]
fn invoke_method(
    method: &js_sys::Function,
    receiver: &JsValue,
    data: &JsValue,
) -> Result<PendingShare, ShareError> {
    let result = method
        .call1(receiver, data)
        .map_err(ShareError::Invocation)?;
    let promise = result
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| ShareError::InvalidPromise)?;
    Ok(PendingShare {
        promise,
        operation: ShareOperation::WebShare,
    })
}

#[cfg(feature = "hydrate")]
fn invoke_share(url: &str, title: &str) -> Result<PendingShare, ShareError> {
    let navigator = web_sys::window()
        .ok_or(ShareError::BrowserUnavailable)?
        .navigator();
    if let Some(method) = find_method(navigator.as_ref(), "share")? {
        let data = web_sys::ShareData::new();
        data.set_title(title);
        data.set_url(url);
        return invoke_method(&method, navigator.as_ref(), data.as_ref());
    }
    let promise = write_clipboard_text(url).map_err(ShareError::Clipboard)?;
    Ok(PendingShare {
        promise,
        operation: ShareOperation::Clipboard,
    })
}

#[cfg(feature = "hydrate")]
fn capture_share_target() -> Result<(String, String), ShareError> {
    let window = web_sys::window().ok_or(ShareError::BrowserUnavailable)?;
    let url = window
        .location()
        .href()
        .map_err(ShareError::LinkUnavailable)?;
    let document = window.document().ok_or(ShareError::BrowserUnavailable)?;
    Ok((url, document.title()))
}

#[cfg(feature = "hydrate")]
fn finish_share(state: ShareState, url: String, outcome: Result<ShareOperation, ShareError>) {
    state.is_sharing.set(false);
    match outcome {
        Ok(ShareOperation::WebShare) => state.feedback.set(Some(ShareFeedback::Invoked)),
        Ok(ShareOperation::Clipboard) => state.feedback.set(Some(ShareFeedback::Copied)),
        Err(error) if error.is_cancelled() => state.feedback.set(Some(ShareFeedback::Cancelled)),
        Err(error) => {
            web_sys::console::error_1(
                &format!(
                    "Share operation failed: {} ({})",
                    error.category(),
                    error.reason_kind()
                )
                .into(),
            );
            if matches!(
                &error,
                ShareError::LinkUnavailable(_) | ShareError::BrowserUnavailable
            ) {
                state.feedback.set(Some(ShareFeedback::LinkUnavailable));
            } else {
                state.panel.set(Some(ManualPanel {
                    url,
                    reason: error.manual_reason(),
                }));
            }
        }
    }
}

#[cfg(feature = "hydrate")]
fn start_share(
    state: ShareState,
    counter: Arc<AtomicU64>,
    is_alive: Arc<AtomicBool>,
    location: Location,
) {
    if state.is_sharing.get_untracked() {
        return;
    }
    state.feedback.set(None);
    state.panel.set(None);
    let (url, title) = match capture_share_target() {
        Ok(target) => target,
        Err(error) => {
            finish_share(state, String::new(), Err(error));
            return;
        }
    };
    state.reset_for_url(url.clone(), &counter);
    let route = route_key(&location);
    let generation = counter.fetch_add(1, Ordering::Relaxed) + 1;
    state.is_sharing.set(true);
    // The API call must happen in this click handler, before any await loses user activation.
    let attempt = invoke_share(&url, &title);
    match attempt {
        Ok(PendingShare { promise, operation }) => wasm_bindgen_futures::spawn_local(async move {
            let result = wasm_bindgen_futures::JsFuture::from(promise)
                .await
                .map(|_| operation)
                .map_err(|source| match operation {
                    ShareOperation::WebShare => ShareError::Invocation(source),
                    ShareOperation::Clipboard => {
                        ShareError::Clipboard(ClipboardError::Rejected(source))
                    }
                });
            if is_current_request(&url, &route, &location, generation, &counter, &is_alive) {
                finish_share(state, url, result);
            }
        }),
        Err(error) => {
            if is_current_request(&url, &route, &location, generation, &counter, &is_alive) {
                finish_share(state, url, Err(error));
            }
        }
    }
}

#[cfg(feature = "hydrate")]
fn observe_navigation(state: ShareState, counter: &AtomicU64) {
    if let Some(url) = web_sys::window().and_then(|window| window.location().href().ok()) {
        state.reset_for_url(url, counter);
    }
}

#[cfg(feature = "hydrate")]
fn watch_navigation(state: ShareState, location: Location) -> (Arc<AtomicU64>, Arc<AtomicBool>) {
    let counter = Arc::new(AtomicU64::new(0));
    let is_alive = Arc::new(AtomicBool::new(true));
    state.reset_for_route(route_key(&location), &counter);
    observe_navigation(state, &counter);
    let reactive_counter = counter.clone();
    Effect::new(move || {
        let route = (
            location.pathname.get(),
            location.search.get(),
            location.hash.get(),
        );
        state.reset_for_route(route, &reactive_counter);
        observe_navigation(state, &reactive_counter);
    });
    let hash_counter = counter.clone();
    let hash_listener = window_event_listener(leptos::ev::hashchange, move |_| {
        observe_navigation(state, &hash_counter);
    });
    let pop_counter = counter.clone();
    let pop_listener = window_event_listener(leptos::ev::popstate, move |_| {
        observe_navigation(state, &pop_counter);
    });
    on_cleanup({
        let counter = counter.clone();
        let is_alive = is_alive.clone();
        move || {
            is_alive.store(false, Ordering::Relaxed);
            counter.fetch_add(1, Ordering::Relaxed);
            hash_listener.remove();
            pop_listener.remove();
        }
    });
    (counter, is_alive)
}

#[component]
fn SharePanel(
    state: ShareState,
    locale: RwSignal<Locale>,
    button: NodeRef<html::Button>,
) -> impl IntoView {
    #[cfg(not(feature = "hydrate"))]
    let _ = &button;
    let input = NodeRef::<html::Input>::new();
    #[cfg(feature = "hydrate")]
    Effect::new(move || {
        if let Some(field) = input.get() {
            let _ = field.focus();
            field.select();
        }
    });
    let close = move || {
        state.panel.set(None);
        #[cfg(feature = "hydrate")]
        if let Some(button) = button.get_untracked() {
            let _ = button.focus();
        }
    };
    view! {
        <div id="share-manual-panel" class="share-panel" on:keydown=move |event| {
            if event.key() == "Escape" {
                event.prevent_default();
                event.stop_propagation();
                close();
            }
        }>
            <p role="alert">{move || match state.panel.get().map(|panel| panel.reason) {
                Some(ManualReason::ShareFailed) => locale.get().select("分享失败，请手动复制链接", "Sharing failed. Copy this link manually."),
                Some(ManualReason::ClipboardFailed) => locale.get().select("无法复制链接，请手动复制链接", "Unable to copy. Copy this link manually."),
                None => "",
            }}</p>
            <label for="share-manual-url">{move || locale.get().select("当前页面链接", "Current page link")}</label>
            <input id="share-manual-url" node_ref=input type="text" readonly value=move || state.panel.get().map(|panel| panel.url).unwrap_or_default()/>
            <button class="share-panel-close" type="button" on:click=move |_| close()>
                {move || locale.get().select("关闭", "Close")}
            </button>
        </div>
    }
}

fn dismiss_feedback(state: ShareState, button: NodeRef<html::Button>) {
    state.feedback.set(None);
    #[cfg(feature = "hydrate")]
    if let Some(button) = button.get_untracked() {
        let _ = button.focus();
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = button;
}

#[component]
fn ShareStatus(
    state: ShareState,
    locale: RwSignal<Locale>,
    button: NodeRef<html::Button>,
) -> impl IntoView {
    let has_status = move || {
        state.panel.get().is_none() && (state.is_sharing.get() || state.feedback.get().is_some())
    };
    let status_text = move || {
        if state.panel.get().is_some() {
            return "";
        }
        let language = locale.get();
        if state.is_sharing.get() {
            return language.select("正在分享当前页面…", "Sharing this page…");
        }
        match state.feedback.get() {
            Some(ShareFeedback::Invoked) => {
                language.select("已调用系统分享", "System sharing invoked")
            }
            Some(ShareFeedback::Copied) => language.select(
                "链接已复制，可以粘贴分享",
                "Link copied. Paste it to share.",
            ),
            Some(ShareFeedback::Cancelled) => language.select("已取消分享", "Sharing cancelled"),
            Some(ShareFeedback::LinkUnavailable) => language.select(
                "无法获取当前页面链接",
                "Unable to get the current page link",
            ),
            None => "",
        }
    };
    view! {
        <span class="sr-only" role="status" aria-live="polite" aria-atomic="true">{status_text}</span>
        <div class="share-status" hidden=move || !has_status()>
            <p aria-hidden="true">{status_text}</p>
            <Show when=move || state.feedback.get().is_some()>
                <button class="share-panel-close" type="button"
                    on:click=move |_| dismiss_feedback(state, button)>
                    {move || locale.get().select("关闭提示", "Dismiss")}
                </button>
            </Show>
        </div>
    }
}

#[component]
pub(super) fn ShareButton(is_hydrated: RwSignal<bool>) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let state = ShareState {
        is_sharing: RwSignal::new(false),
        feedback: RwSignal::new(None),
        panel: RwSignal::new(None),
        #[cfg(feature = "hydrate")]
        current_url: RwSignal::new(String::new()),
        #[cfg(feature = "hydrate")]
        current_route: RwSignal::new(None),
    };
    let button = NodeRef::<html::Button>::new();
    #[cfg(feature = "hydrate")]
    let location = use_location();
    #[cfg(feature = "hydrate")]
    let (counter, is_alive) = watch_navigation(state, location.clone());
    view! {
        <div class="share-action" on:keydown=move |event| {
            if event.key() == "Escape" && state.feedback.get_untracked().is_some() {
                event.prevent_default();
                event.stop_propagation();
                dismiss_feedback(state, button);
            }
        }>
            <button
                node_ref=button
                class="topbar-button share-button"
                type="button"
                disabled=move || !is_hydrated.get() || state.is_sharing.get()
                aria-label=move || locale.get().select("分享当前页面", "Share this page")
                title=move || locale.get().select("分享当前页面：支持时打开系统分享，否则复制链接", "Share this page: open system sharing when available, otherwise copy the link")
                aria-describedby="share-description"
                aria-expanded=move || if state.panel.get().is_some() { "true" } else { "false" }
                aria-controls="share-manual-panel"
                on:click=move |_| {
                    #[cfg(feature = "hydrate")]
                    start_share(state, counter.clone(), is_alive.clone(), location.clone());
                }
            ><Icon kind=IconKind::Share/></button>
            <span id="share-description" class="sr-only">{move || locale.get().select(
                "分享当前页面：支持时打开系统分享，否则复制链接",
                "Share this page: open system sharing when available, otherwise copy the link",
            )}</span>
            <ShareStatus state locale button/>
            <Show when=move || state.panel.get().is_some()>
                <SharePanel state locale button/>
            </Show>
        </div>
    }
}
