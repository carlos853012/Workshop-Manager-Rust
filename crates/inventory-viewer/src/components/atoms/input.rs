use dioxus::prelude::*;

#[component]
pub fn Input(
    value: String,
    oninput: EventHandler<FormEvent>,
    #[props(default = "text".to_string())] r#type: String,
    placeholder: Option<String>,
    label: Option<String>,
    error: Option<String>,
    #[props(default = false)] disabled: bool,
    #[props(default = false)] required: bool,
    class: Option<String>,
) -> Element {
    let input_type = r#type;
    let mut classes = vec!["input".to_string()];
    if error.is_some() {
        classes.push("input-error".to_string());
    }
    if let Some(extra) = class {
        classes.push(extra);
    }
    let class_str = classes.join(" ");

    rsx! {
        div { class: "form-group",
            if let Some(label_text) = label {
                label { class: "form-label", "{label_text}" }
            }
            input {
                class: "{class_str}",
                r#type: "{input_type}",
                value: "{value}",
                placeholder: placeholder.unwrap_or_default(),
                disabled: disabled,
                required: required,
                oninput: move |evt| oninput.call(evt),
            }
            if let Some(err) = error {
                span { class: "form-error", "{err}" }
            }
        }
    }
}
