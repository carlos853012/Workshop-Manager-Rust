use dioxus::prelude::*;

#[component]
pub fn Modal(
    children: Element,
    title: String,
    #[props(default = true)] show: bool,
    on_close: EventHandler<()>,
    footer: Option<Element>,
    #[props(default = None)] class: Option<String>,
) -> Element {
    if !show {
        return rsx! {};
    }

    let modal_class = class
        .as_deref()
        .map(|c| format!("modal {}", c))
        .unwrap_or_else(|| "modal".to_string());

    rsx! {
        div {
            class: "modal-backdrop",
            onclick: move |_evt| on_close.call(()),
            div {
                class: "{modal_class}",
                onclick: move |evt| evt.stop_propagation(),

                div { class: "modal-body", {children} }
                if let Some(footer_content) = footer {
                    div { class: "modal-footer", {footer_content} }
                }
            }
        }
    }
}
