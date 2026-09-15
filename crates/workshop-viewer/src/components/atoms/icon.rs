use dioxus::prelude::*;

/// Renderiza un SVG inline de 24x24.
#[component]
pub fn Icon(
    svg: String,
    #[props(default = "1.25rem".to_string())] size: String,
    class: Option<String>,
) -> Element {
    let class_str = class.unwrap_or_default();
    rsx! {
        span {
            class: "{class_str}",
            style: "display:inline-flex; width:{size}; height:{size};",
            dangerous_inner_html: "{svg}"
        }
    }
}
