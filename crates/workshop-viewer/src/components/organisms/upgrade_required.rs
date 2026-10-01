use dioxus::prelude::*;

use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::organisms::license_activation_modal::LicenseActivationModal;
use crate::i18n;
use crate::icons::IconName;

/// Pantalla mostrada cuando el módulo actual no está incluido en la licencia.
#[component]
pub fn UpgradeRequired(module: String) -> Element {
    let mut show_modal = use_signal(|| false);

    rsx! {
        div { class: "empty-state",
            {IconName::Ban.render()}
            h2 { class: "mt-lg", {i18n::blocked_module_title()} }
            p { class: "text-muted mb-md", {i18n::blocked_module_message(&module)} }
            Button {
                variant: ButtonVariant::Primary,
                onclick: move |_| show_modal.set(true),
                {i18n::blocked_module_cta()}
            }
        }
        LicenseActivationModal {
            show: *show_modal.read(),
            on_close: move |_| show_modal.set(false),
        }
    }
}
