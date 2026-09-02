use dioxus::prelude::*;

/// Columna genérica de la tabla.
#[derive(Clone)]
pub struct Column<T: Clone + 'static> {
    pub key: String,
    pub header: String,
    pub render: fn(&T) -> Element,
}

impl<T: Clone + 'static> PartialEq for Column<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.header == other.header
    }
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
        total.div_ceil(per_page)
    };

    rsx! {
        div { class: "data-table-wrapper",
            table { class: "data-table",
                thead {
                    tr {
                        for col in columns.iter() {
                            th { "{col.header}" }
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
                                    td { { (col.render)(row) } }
                                }
                            }
                        }
                    }
                }
            }
            if total_pages > 1 {
                div { class: "pagination",
                    button {
                        class: "btn btn-ghost",
                        disabled: page <= 1,
                        onclick: move |_evt| {
                            if let Some(handler) = on_page_change.as_ref() {
                                handler.call(page.saturating_sub(1));
                            }
                        },
                        "Anterior"
                    }
                    span { class: "text-muted", "Página {page} de {total_pages}" }
                    button {
                        class: "btn btn-ghost",
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
