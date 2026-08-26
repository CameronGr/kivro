use std::cell::Cell;
use std::time::Duration;

use leptos::ev;
use leptos::html::Div;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

pub const SCROLL_OFFSET_PX: f64 = 8.0 * 16.0;

pub fn use_click_outside(node: NodeRef<Div>, on_outside: impl Fn() + 'static) {
    let handle = window_event_listener(ev::mousedown, move |ev| {
        let Some(root) = node.get_untracked() else {
            return;
        };
        let inside = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Node>().ok())
            .is_some_and(|t| root.contains(Some(&t)));

        if !inside {
            on_outside();
        }
    });
    on_cleanup(move || handle.remove());
}

pub fn use_escape(on_escape: impl Fn() + 'static) {
    let handle = window_event_listener(ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            on_escape();
        }
    });
    on_cleanup(move || handle.remove());
}

pub fn use_clipboard(reset_after: Duration) -> (Signal<bool>, impl Fn(String) + Copy + 'static) {
    let copied = RwSignal::new(false);
    let generation = StoredValue::new_local(Cell::new(0u32));

    let copy = move |text: String| {
        write_clipboard(&text);
        let genr = generation.with_value(|g| {
            let next = g.get().wrapping_add(1);
            g.set(next);
            next
        });

        copied.set(true);
        set_timeout(
            move || {
                if generation.with_value(|g| g.get()) == genr {
                    copied.set(false);
                }
            },
            reset_after,
        );
    };

    (copied.into(), copy)
}

pub fn write_clipboard(text: &str) {
    if let Some(win) = web_sys::window() {
        let _ = win.navigator().clipboard().write_text(text);
    }
}

pub fn scroll_container(el: &web_sys::Element) -> web_sys::Element {
    let doc = web_sys::window().and_then(|w| w.document());
    let body = doc.as_ref().and_then(|d| d.body());
    let mut node = el.parent_element();

    while let Some(current) = node {
        if body
            .as_ref()
            .is_some_and(|b| b.is_same_node(Some(&current)))
        {
            break;
        }
        let overflow = web_sys::window()
            .and_then(|w| w.get_computed_style(&current).ok().flatten())
            .and_then(|s| s.get_property_value("overflow-y").ok())
            .unwrap_or_default();
        if overflow == "auto" || overflow == "scroll" {
            return current;
        }
        node = current.parent_element();
    }
    doc.and_then(|d| d.document_element())
        .unwrap_or_else(|| el.clone())
}

pub fn scroll_to_id(slug: &str, instant: bool) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(el) = doc.get_element_by_id(slug) else {
        return;
    };
    let container = scroll_container(&el);
    let target = container.scroll_top() as f64
        + (el.get_bounding_client_rect().top() - container.get_bounding_client_rect().top())
        - SCROLL_OFFSET_PX;

    let opts = web_sys::ScrollToOptions::new();
    opts.set_top(target.max(0.0));
    opts.set_behavior(if instant {
        web_sys::ScrollBehavior::Instant
    } else {
        web_sys::ScrollBehavior::Smooth
    });
    container.scroll_to_with_scroll_to_options(&opts);
}

pub fn use_active_section(slugs: Signal<Vec<String>>) -> RwSignal<Option<String>> {
    let active = RwSignal::new(None::<String>);
    let keepalive = StoredValue::new_local(
        None::<(
            web_sys::IntersectionObserver,
            Closure<dyn FnMut(js_sys::Array)>,
        )>,
    );

    Effect::new(move |_| {
        let slugs = slugs.get();

        keepalive.update_value(|slot| {
            if let Some((observer, _)) = slot.take() {
                observer.disconnect();
            }
        });

        if slugs.is_empty() {
            return;
        }
        let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
            return;
        };

        let callback = Closure::<dyn FnMut(js_sys::Array)>::new(move |entries: js_sys::Array| {
            let mut best: Option<(f64, String)> = None;
            for entry in entries.iter() {
                let Ok(entry) = entry.dyn_into::<web_sys::IntersectionObserverEntry>() else {
                    continue;
                };
                if !entry.is_intersecting() {
                    continue;
                }
                let ratio = entry.intersection_ratio();
                let id = entry.target().id();
                if id.is_empty() {
                    continue;
                }
                if best.as_ref().is_none_or(|(r, _)| ratio > *r) {
                    best = Some((ratio, id));
                }
            }
            if let Some((_, id)) = best {
                active.set(Some(id));
            }
        });

        let init = web_sys::IntersectionObserverInit::new();
        init.set_root_margin("-15% 0px -65% 0px");
        init.set_threshold(&js_sys::Array::of4(
            &0.1.into(),
            &0.25.into(),
            &0.4.into(),
            &0.65.into(),
        ));

        let Ok(observer) = web_sys::IntersectionObserver::new_with_options(
            callback.as_ref().unchecked_ref(),
            &init,
        ) else {
            return;
        };

        for slug in &slugs {
            if let Some(el) = doc.get_element_by_id(slug) {
                observer.observe(&el);
            }
        }
        if active.get_untracked().is_none() {
            active.set(slugs.first().cloned());
        }
        keepalive.set_value(Some((observer, callback)));
    });

    on_cleanup(move || {
        keepalive.update_value(|slot| {
            if let Some((observer, _)) = slot.take() {
                observer.disconnect();
            }
        });
    });

    active
}

pub fn use_id(prefix: &str) -> String {
    thread_local! {
        static COUNTER: Cell<u64> = const {Cell::new(0)};
    }
    let n = COUNTER.with(|c| {
        let next = c.get().wrapping_add(1);
        c.set(next);
        next
    });
    format!("iui-{prefix}-{n}")
}

pub fn use_scroll_lock(locked: Signal<bool>) {
    let previous = StoredValue::new_local(None::<String>);

    let restore = move || {
        let Some(style) = body_style() else { return };
        let prior = previous.with_value(|p| p.clone());
        if let Some(prior) = prior {
            let _ = style.set_property("overflow", &prior);
            previous.set_value(None);
        }
    };

    Effect::new(move |_| {
        let Some(style) = body_style() else {
            return;
        };
        if locked.get() {
            if previous.with_value(|p| p.is_none()) {
                let current = style.get_property_value("overflow").unwrap_or_default();
                previous.set_value(Some(current));
            }
            let _ = style.set_property("overflow", "hidden");
        } else {
            restore();
        }
    });

    on_cleanup(restore);
}

fn body_style() -> Option<web_sys::CssStyleDeclaration> {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body())
        .map(|b| b.style())
}
