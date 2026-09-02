use dioxus::prelude::*;

use crate::components::atoms::button::{Button, ButtonVariant};

/// Columna genérica de la tabla.
#[derive(Clone, PartialEq)]
pub struct Column<T: Clone + PartialEq + 'static> {
    pub key: String,
    pub header: String,
    pub render: fn(&T) -> Element,
}

#[component]
pub fn DataTable<T: Clone + PartialEq + 'static>(
    columns: Vec<Column<T>>,
    rows: Vec<T>,
    #[props(default = 1)] page: usize,
    #[props(default = 10)] per_page: usize,
    #[props(default = 0)] total: usize,
    on_page_change: Option<EventHandler<usize>>,
) -> Element {
    let total_pages = if total == 0 {
        1
    } else {
        (total + per_page - 1) / per_page
    };

    rsx! {
        div { class: "data-table-wrapper",
            table { class: "data-table",
                thead {
                    tr {
                        for col in columns.iter() {
                            th { key: "{col.key}", "{col.header}" }
                        }
                    }
                }
                tbody {
                    if rows.is_empty() {
                        tr {
                            td {
                                colspan: "{columns.len()}",
                                div { class: "empty-state", "No hay datos para mostrar" }
                            }
                        }
                    } else {
                        for row in rows.iter() {
                            tr {
                                for col in columns.iter() {
                                    td { key: "{col.key}", (col.render)(row) }
                                }
                            }
                        }
                    }
                }
            }
            if total_pages > 1 {
                div { class: "pagination",
                    Button {
                        variant: ButtonVariant::Ghost,
                        disabled: page <= 1,
                        onclick: move |_evt| {
                            if let Some(handler) = on_page_change.as_ref() {
                                handler.call(page.saturating_sub(1));
                            }
                        },
                        "Anterior"
                    }
                    span { class: "text-muted", "Página {page} de {total_pages}" }
                    Button {
                        variant: ButtonVariant::Ghost,
                        disabled: page >= total_pages,
                        onclick: move |_evt| {
                            if let Some(handler) = on_page_change.as_ref() {
                                handler.call(page + 1);
                            }
                        },
                        "Siguiente"
                    }
                }
            }
        }
    }
}
