use dioxus::prelude::*;

#[component]
pub fn FormGroup(
    children: Element,
    label: Option<String>,
    error: Option<String>,
    #[props(default = false)] required: bool,
) -> Element {
    rsx! {
        div { class: "form-group",
            if let Some(label_text) = label {
                label { class: "form-label",
                    "{label_text}"
                    if required {
                        span { class: "text-danger", " *" }
                    }
                }
            }
            {children}
            if let Some(err) = error {
                span { class: "form-error", "{err}" }
            }
        }
    }
}
