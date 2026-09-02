use dioxus::prelude::*;

#[component]
pub fn Card(
    children: Element,
    title: Option<String>,
    footer: Option<Element>,
    class: Option<String>,
) -> Element {
    let class_str = format!("card {}", class.unwrap_or_default());
    rsx! {
        div { class: "{class_str}",
            if let Some(title_text) = title {
                div { class: "card-header", "{title_text}" }
            }
            div { class: "card-body", {children} }
            if let Some(footer_content) = footer {
                div { class: "card-footer", {footer_content} }
            }
        }
    }
}
