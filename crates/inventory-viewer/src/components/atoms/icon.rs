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

/// Helper para envolver paths SVG heroicons.
pub fn svg_wrapper(paths: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="100%" height="100%">{}</svg>"#,
        paths
    )
}
