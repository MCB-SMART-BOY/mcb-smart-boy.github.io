//! Wheel gesture handling for flipping between the sidebar's main and reading pages.
//! Kept separate from the navigation view so scroll arbitration is reviewable in isolation.

use leptos::{html, prelude::*};
use leptos_router::NavigateOptions;
use std::cell::Cell;

use super::reading_navigation::{
    NavigationPage, NavigationState, has_focus_within, navigate_root_page,
};
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, Default)]
struct WheelGesture {
    last_at: f64,
    locked_until: f64,
    accumulated: f64,
}

const WHEEL_LINE_MODE: u32 = 1;
const WHEEL_PAGE_MODE: u32 = 2;
const WHEEL_LINE_PX: f64 = 16.0;
const WHEEL_THRESHOLD_PX: f64 = 36.0;
const WHEEL_GESTURE_GAP_MS: f64 = 180.0;
const WHEEL_FLIP_LOCK_MS: f64 = 560.0;

fn is_scrollable_in_direction(top: i32, height: i32, viewport: i32, delta: f64) -> bool {
    let remaining = height - viewport;
    remaining > 1
        && if delta > 0.0 {
            top < remaining - 1
        } else {
            top > 1
        }
}

fn handle_gesture(
    state: NavigationState,
    panel_ref: NodeRef<html::Div>,
    delta: f64,
    now: f64,
    gesture_cell: &Cell<WheelGesture>,
    navigate: impl Fn(&str, NavigateOptions),
) -> bool {
    let Some(panel) = panel_ref.get_untracked() else {
        return false;
    };
    let mut gesture = gesture_cell.get();
    if now < gesture.locked_until
        || (gesture.locked_until > 0.0 && now - gesture.last_at < WHEEL_GESTURE_GAP_MS)
    {
        gesture.last_at = now;
        gesture_cell.set(gesture);
        return true;
    }
    if is_scrollable_in_direction(
        panel.scroll_top(),
        panel.scroll_height(),
        panel.client_height(),
        delta,
    ) {
        gesture_cell.set(WheelGesture::default());
        return false;
    }
    gesture.locked_until = 0.0;
    if now - gesture.last_at > WHEEL_GESTURE_GAP_MS
        || gesture.accumulated.signum() != delta.signum()
    {
        gesture.accumulated = 0.0;
    }
    gesture.last_at = now;
    gesture.accumulated += delta;
    if gesture.accumulated.abs() >= WHEEL_THRESHOLD_PX {
        let next = if delta > 0.0 {
            NavigationPage::Reading
        } else {
            NavigationPage::Main
        };
        navigate_root_page(state, next, has_focus_within(panel_ref), &navigate);
        gesture.locked_until = now + WHEEL_FLIP_LOCK_MS;
        gesture.accumulated = 0.0;
    }
    gesture_cell.set(gesture);
    true
}

pub(super) fn install(
    container: NodeRef<html::Div>,
    state: NavigationState,
    is_visible: Signal<bool>,
    panels: [NodeRef<html::Div>; NavigationPage::COUNT],
    navigate: impl Fn(&str, NavigateOptions) + Clone + 'static,
) {
    let listener = StoredValue::new_local(std::cell::RefCell::new(
        None::<(
            web_sys::HtmlElement,
            wasm_bindgen::closure::Closure<dyn FnMut(leptos::ev::WheelEvent)>,
        )>,
    ));
    let gesture = StoredValue::new_local(Cell::new(WheelGesture::default()));
    Effect::new(move |_| {
        if let Some((element, callback)) = listener.with_value(|slot| slot.borrow_mut().take()) {
            if let Err(source) = element
                .remove_event_listener_with_callback("wheel", callback.as_ref().unchecked_ref())
            {
                web_sys::console::error_2(
                    &"Failed to remove reading navigation wheel listener".into(),
                    &source,
                );
            }
        }
        let Some(element) = container.get() else {
            return;
        };
        let element: web_sys::HtmlElement = element.into();
        let navigate = navigate.clone();
        let callback = wasm_bindgen::closure::Closure::<dyn FnMut(leptos::ev::WheelEvent)>::new(
            move |event: leptos::ev::WheelEvent| {
                if !is_visible.get_untracked()
                    || event.ctrl_key()
                    || event.meta_key()
                    || event.alt_key()
                    || event.shift_key()
                {
                    return;
                }
                let page = state.page.get_untracked();
                let panel = panels[page.index()];
                let delta = match event.delta_mode() {
                    WHEEL_LINE_MODE => event.delta_y() * WHEEL_LINE_PX,
                    WHEEL_PAGE_MODE => {
                        event.delta_y()
                            * f64::from(
                                panel
                                    .get_untracked()
                                    .map_or(0, |panel| panel.client_height()),
                            )
                    }
                    _ => event.delta_y(),
                };
                if delta.abs() < f64::EPSILON
                    || (page == NavigationPage::Reading && !state.is_index.get_untracked())
                {
                    return;
                }
                let gesture_state = gesture.with_value(|cell| cell.get());
                let gesture_locked = gesture_state.locked_until > event.time_stamp();
                if !gesture_locked
                    && ((page == NavigationPage::Main && delta < 0.0)
                        || (page == NavigationPage::Reading && delta > 0.0))
                {
                    return;
                }
                let handled = gesture.with_value(|cell| {
                    handle_gesture(state, panel, delta, event.time_stamp(), cell, &navigate)
                });
                if handled {
                    event.prevent_default();
                }
            },
        );
        let options = web_sys::AddEventListenerOptions::new();
        options.set_passive(false);
        if let Err(source) = element
            .add_event_listener_with_callback_and_add_event_listener_options(
                "wheel",
                callback.as_ref().unchecked_ref(),
                &options,
            )
        {
            web_sys::console::error_2(
                &"Failed to attach reading navigation wheel listener".into(),
                &source,
            );
            return;
        }
        listener.with_value(|slot| *slot.borrow_mut() = Some((element, callback)));
    });
    on_cleanup(move || {
        if let Some((element, callback)) = listener.with_value(|slot| slot.borrow_mut().take()) {
            if let Err(source) = element
                .remove_event_listener_with_callback("wheel", callback.as_ref().unchecked_ref())
            {
                web_sys::console::error_2(
                    &"Failed to remove reading navigation wheel listener".into(),
                    &source,
                );
            }
        }
    });
}
