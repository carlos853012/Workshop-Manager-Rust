use dioxus::prelude::*;

#[component]
pub fn Tooltip(children: Element, text: String) -> Element {
    rsx! {
        span {
            class: "tooltip",
            style: "position: relative; display: inline-flex; cursor: help;",
            {children}
            span {
                class: "tooltip-text",
                style: "position: absolute; bottom: 100%; left: 50%; transform: translateX(-50%); white-space: nowrap; padding: 0.25rem 0.5rem; background: var(--text); color: var(--bg); border-radius: var(--radius-sm); font-size: var(--font-size-sm); opacity: 0; pointer-events: none; transition: opacity 0.15s;",
                "{text}"
            }
        }
    }
}
