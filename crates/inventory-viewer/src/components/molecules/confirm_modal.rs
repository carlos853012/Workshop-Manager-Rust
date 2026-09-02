use dioxus::prelude::*;

use crate::components::atoms::button::{Button, ButtonVariant};

#[component]
pub fn ConfirmModal(
    title: String,
    message: String,
    #[props(default = true)] show: bool,
    confirm_text: Option<String>,
    cancel_text: Option<String>,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    if !show {
        return rsx! {};
    }

    let confirm = confirm_text.unwrap_or_else(|| "Confirmar".to_string());
    let cancel = cancel_text.unwrap_or_else(|| "Cancelar".to_string());

    rsx! {
        div {
            class: "modal-backdrop",
            onclick: move |_evt| on_cancel.call(()),
            div {
                class: "modal",
                onclick: move |evt| evt.stop_propagation(),
                div { class: "modal-header",
                    h3 { "{title}" }
                    button {
                        class: "btn btn-ghost btn-sm",
                        onclick: move |_evt| on_cancel.call(()),
                        "✕"
                    }
                }
                div { class: "modal-body",
                    p { "{message}" }
                }
                div { class: "modal-footer",
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: move |_evt| on_cancel.call(()),
                        "{cancel}"
                    }
                    Button {
                        variant: ButtonVariant::Danger,
                        onclick: move |_evt| on_confirm.call(()),
                        "{confirm}"
                    }
                }
            }
        }
    }
}
