use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::clipboard::{ClipboardError, classify_js_error};

#[cfg(feature = "hydrate")]
use leptos_router::location::Location;

#[cfg(feature = "hydrate")]
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

#[cfg(feature = "hydrate")]
use wasm_bindgen::JsValue;

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
#[derive(Clone, Copy)]
pub(super) enum ShareFeedback {
    Invoked,
    Copied,
    Cancelled,
    LinkUnavailable,
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
#[derive(Clone, Copy)]
pub(super) enum ManualReason {
    ShareFailed,
    ClipboardFailed,
}

#[derive(Clone)]
pub(super) struct ManualPanel {
    pub(super) url: String,
    pub(super) reason: ManualReason,
}

#[derive(Clone, Copy)]
pub(super) struct ShareState {
    pub(super) is_sharing: RwSignal<bool>,
    pub(super) feedback: RwSignal<Option<ShareFeedback>>,
    pub(super) panel: RwSignal<Option<ManualPanel>>,
    #[cfg(feature = "hydrate")]
    pub(super) current_url: RwSignal<String>,
    #[cfg(feature = "hydrate")]
    pub(super) current_route: RwSignal<Option<(String, String, String)>>,
}

#[cfg(feature = "hydrate")]
impl ShareState {
    pub(super) fn reset_for_url(self, url: String, generation: &AtomicU64) {
        if self.current_url.get_untracked() == url {
            return;
        }
        self.current_url.set(url);
        self.invalidate(generation);
    }

    pub(super) fn reset_for_route(self, route: (String, String, String), generation: &AtomicU64) {
        if self.current_route.get_untracked().as_ref() == Some(&route) {
            return;
        }
        self.current_route.set(Some(route));
        self.invalidate(generation);
    }

    fn invalidate(self, generation: &AtomicU64) {
        generation.fetch_add(1, Ordering::Relaxed);
        self.is_sharing.set(false);
        self.feedback.set(None);
        self.panel.set(None);
    }
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Copy)]
pub(super) enum ShareOperation {
    WebShare,
    Clipboard,
}

#[cfg(feature = "hydrate")]
pub(super) enum ShareError {
    BrowserUnavailable,
    LinkUnavailable(JsValue),
    Invocation(JsValue),
    InvalidPromise,
    InvalidMethod,
    Clipboard(ClipboardError),
}

#[cfg(feature = "hydrate")]
impl ShareError {
    pub(super) fn is_cancelled(&self) -> bool {
        matches!(self, Self::Invocation(source) if classify_js_error(source) == "AbortError")
    }

    pub(super) fn category(&self) -> &'static str {
        match self {
            Self::BrowserUnavailable => "browser unavailable",
            Self::LinkUnavailable(_) => "location unavailable",
            Self::Invocation(_) => "web share rejected",
            Self::InvalidPromise => "web share returned no promise",
            Self::InvalidMethod => "web share method invalid",
            Self::Clipboard(error) => error.category(),
        }
    }

    pub(super) fn reason_kind(&self) -> &'static str {
        match self {
            Self::LinkUnavailable(source) | Self::Invocation(source) => classify_js_error(source),
            Self::Clipboard(error) => error.reason_kind(),
            _ => "none",
        }
    }

    pub(super) fn manual_reason(&self) -> ManualReason {
        match self {
            Self::Clipboard(_) => ManualReason::ClipboardFailed,
            _ => ManualReason::ShareFailed,
        }
    }
}

#[cfg(feature = "hydrate")]
pub(super) struct PendingShare {
    pub(super) promise: js_sys::Promise,
    pub(super) operation: ShareOperation,
}

#[cfg(feature = "hydrate")]
pub(super) fn route_key(location: &Location) -> (String, String, String) {
    (
        location.pathname.get_untracked(),
        location.search.get_untracked(),
        location.hash.get_untracked(),
    )
}

#[cfg(feature = "hydrate")]
pub(super) fn is_current_request(
    url: &str,
    route: &(String, String, String),
    location: &Location,
    generation: u64,
    counter: &Arc<AtomicU64>,
    is_alive: &Arc<AtomicBool>,
) -> bool {
    is_alive.load(Ordering::Relaxed)
        && counter.load(Ordering::Relaxed) == generation
        && route_key(location) == *route
        && web_sys::window()
            .and_then(|window| window.location().href().ok())
            .as_deref()
            == Some(url)
}
