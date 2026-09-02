use dioxus::prelude::*;

#[component]
pub fn Modal(
    children: Element,
    title: String,
    #[props(default = true)] show: bool,
    on_close: EventHandler<()>,
    footer: Option<Element>,
) -> Element {
    if !show {
        return rsx! {};
    }

    rsx! {
        div {
            class: "modal-backdrop",
            onclick: move |_evt| on_close.call(()),
            div {
                class: "modal",
                onclick: move |evt| evt.stop_propagation(),
                div { class: "modal-header",
                    h3 { "{title}" }
                    button {
                        class: "btn btn-ghost btn-sm",
                        onclick: move |_evt| on_close.call(()),
                        "✕"
                    }
                }
                div { class: "modal-body", {children} }
                if let Some(footer_content) = footer {
                    div { class: "modal-footer", {footer_content} }
                }
            }
        }
    }
}
