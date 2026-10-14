use wasm_bindgen::{JsCast, JsValue};

/// Failure while resolving or invoking the browser Clipboard API.
pub(crate) enum ClipboardError {
    /// The browser window is unavailable.
    BrowserUnavailable,
    /// The browser has no accessible Clipboard object.
    ClipboardUnavailable,
    /// Accessing the Clipboard object or its method threw a JS exception.
    Access(JsValue),
    /// The Clipboard object does not expose writeText.
    MethodUnavailable,
    /// The exposed writeText property is not callable; retains its JS value.
    InvalidMethod(JsValue),
    /// Calling writeText threw synchronously; retains the JS exception.
    Invocation(JsValue),
    /// writeText did not return a Promise; retains its JS value.
    InvalidPromise(JsValue),
    /// The writeText Promise rejected; retains the JS rejection.
    Rejected(JsValue),
}

impl ClipboardError {
    pub(crate) fn category(&self) -> &'static str {
        match self {
            Self::BrowserUnavailable => "browser unavailable",
            Self::ClipboardUnavailable | Self::MethodUnavailable => "clipboard unavailable",
            Self::Access(_) => "clipboard access failed",
            Self::InvalidMethod(_) => "clipboard method invalid",
            Self::Invocation(_) | Self::Rejected(_) => "clipboard rejected",
            Self::InvalidPromise(_) => "clipboard returned no promise",
        }
    }

    pub(crate) fn reason_kind(&self) -> &'static str {
        match self {
            Self::Access(source)
            | Self::InvalidMethod(source)
            | Self::Invocation(source)
            | Self::InvalidPromise(source)
            | Self::Rejected(source) => classify_js_error(source),
            _ => "none",
        }
    }
}

/// Returns only allowlisted JS error names, never the original JS message.
pub(crate) fn classify_js_error(source: &JsValue) -> &'static str {
    let name = source
        .dyn_ref::<web_sys::DomException>()
        .map(|error| error.name())
        .or_else(|| {
            js_sys::Reflect::get(source, &JsValue::from_str("name"))
                .ok()
                .and_then(|name| name.as_string())
        });
    match name.as_deref() {
        Some("AbortError") => "AbortError",
        Some("NotAllowedError") => "NotAllowedError",
        Some("SecurityError") => "SecurityError",
        Some("TypeError") => "TypeError",
        Some("DataError") => "DataError",
        Some("InvalidStateError") => "InvalidStateError",
        _ => "other",
    }
}

/// Calls writeText immediately so the caller can invoke it inside a user click handler.
pub(crate) fn write_clipboard_text(text: &str) -> Result<js_sys::Promise, ClipboardError> {
    let navigator = web_sys::window()
        .ok_or(ClipboardError::BrowserUnavailable)?
        .navigator();
    let clipboard = js_sys::Reflect::get(navigator.as_ref(), &JsValue::from_str("clipboard"))
        .map_err(ClipboardError::Access)?;
    if clipboard.is_null() || clipboard.is_undefined() {
        return Err(ClipboardError::ClipboardUnavailable);
    }
    let method = js_sys::Reflect::get(&clipboard, &JsValue::from_str("writeText"))
        .map_err(ClipboardError::Access)?;
    if method.is_null() || method.is_undefined() {
        return Err(ClipboardError::MethodUnavailable);
    }
    let method = method
        .dyn_into::<js_sys::Function>()
        .map_err(ClipboardError::InvalidMethod)?;
    let result = method
        .call1(&clipboard, &JsValue::from_str(text))
        .map_err(ClipboardError::Invocation)?;
    result
        .dyn_into::<js_sys::Promise>()
        .map_err(ClipboardError::InvalidPromise)
}
