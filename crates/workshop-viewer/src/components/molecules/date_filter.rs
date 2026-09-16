use dioxus::prelude::*;

use crate::components::atoms::button::{Button, ButtonVariant};

fn today_str() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d").to_string()
}

fn days_ago(n: i64) -> String {
    let now = chrono::Local::now();
    (now - chrono::Duration::days(n))
        .format("%Y-%m-%d")
        .to_string()
}

#[derive(Clone, PartialEq)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

impl Default for DateRange {
    fn default() -> Self {
        Self {
            start: days_ago(180),
            end: today_str(),
        }
    }
}

/// Componente de filtro de fechas con botones rápidos y campos personalizados.
#[component]
pub fn DateFilter(on_change: EventHandler<DateRange>) -> Element {
    let mut active_quick = use_signal(|| Some(2usize));
    let mut start_date = use_signal(|| days_ago(180));
    let mut end_date = use_signal(today_str);

    let today = today_str();
    let r7_start = days_ago(7);
    let r30_start = days_ago(30);
    let r180_start = days_ago(180);
    let year_start = format!("{}-01-01", chrono::Local::now().format("%Y"));

    rsx! {
        div { class: "date-filter",
            div { class: "date-filter-quick",
                Button {
                    variant: if *active_quick.read() == Some(0) { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                    class: Some("date-filter-btn".to_string()),
                    onclick: {
                        let r7 = r7_start.clone();
                        let t = today.clone();
                        move |_| {
                            active_quick.set(Some(0));
                            start_date.set(r7.clone());
                            end_date.set(t.clone());
                            on_change.call(DateRange { start: r7.clone(), end: t.clone() });
                        }
                    },
                    "7 días"
                }
                Button {
                    variant: if *active_quick.read() == Some(1) { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                    class: Some("date-filter-btn".to_string()),
                    onclick: {
                        let r30 = r30_start.clone();
                        let t = today.clone();
                        move |_| {
                            active_quick.set(Some(1));
                            start_date.set(r30.clone());
                            end_date.set(t.clone());
                            on_change.call(DateRange { start: r30.clone(), end: t.clone() });
                        }
                    },
                    "30 días"
                }
                Button {
                    variant: if *active_quick.read() == Some(2) { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                    class: Some("date-filter-btn".to_string()),
                    onclick: {
                        let r180 = r180_start.clone();
                        let t = today.clone();
                        move |_| {
                            active_quick.set(Some(2));
                            start_date.set(r180.clone());
                            end_date.set(t.clone());
                            on_change.call(DateRange { start: r180.clone(), end: t.clone() });
                        }
                    },
                    "6 meses"
                }
                Button {
                    variant: if *active_quick.read() == Some(3) { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                    class: Some("date-filter-btn".to_string()),
                    onclick: {
                        let ys = year_start.clone();
                        let t = today.clone();
                        move |_| {
                            active_quick.set(Some(3));
                            start_date.set(ys.clone());
                            end_date.set(t.clone());
                            on_change.call(DateRange { start: ys.clone(), end: t.clone() });
                        }
                    },
                    "Este año"
                }
            }
            div { class: "date-filter-custom",
                span { class: "date-filter-label", "Desde" }
                input {
                    r#type: "date",
                    class: "date-filter-input",
                    value: "{start_date}",
                    onchange: move |evt: Event<FormData>| {
                        start_date.set(evt.value());
                        active_quick.set(None);
                        on_change.call(DateRange {
                            start: start_date.read().clone(),
                            end: end_date.read().clone(),
                        });
                    },
                }
                span { class: "date-filter-label", "Hasta" }
                input {
                    r#type: "date",
                    class: "date-filter-input",
                    value: "{end_date}",
                    onchange: move |evt: Event<FormData>| {
                        end_date.set(evt.value());
                        active_quick.set(None);
                        on_change.call(DateRange {
                            start: start_date.read().clone(),
                            end: end_date.read().clone(),
                        });
                    },
                }
            }
        }
    }
}
