use dioxus::prelude::*;

#[component]
pub fn Spinner(#[props(default = "1.25rem".to_string())] size: String) -> Element {
    rsx! {
        span {
            class: "spinner",
            style: "width:{size}; height:{size};"
        }
    }
}
