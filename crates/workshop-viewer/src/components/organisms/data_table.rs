use dioxus::prelude::*;
use std::rc::Rc;

/// Columna genérica de la tabla.
#[derive(Clone)]
pub struct Column<T: Clone + 'static> {
    pub key: String,
    pub header: String,
    pub render: Rc<dyn Fn(&T) -> Element>,
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
    #[props(default = 1)] page: i32,
    #[props(default = 10)] per_page: i32,
    #[props(default = 0)] total: i64,
    #[props(default = EventHandler::new(|_| {}))] on_page_change: EventHandler<i32>,
) -> Element {
    let total_pages: i64 = if total == 0 {
        1
    } else {
        let per_page_i64 = i64::from(per_page);
        (total + per_page_i64 - 1) / per_page_i64
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
                        disabled: i64::from(page) <= 1,
                        onclick: move |_evt| on_page_change.call(page.saturating_sub(1)),
                        "Anterior"
                    }
                    span { class: "text-muted", "Página {page} de {total_pages}" }
                    button {
                        class: "btn btn-ghost",
                        disabled: i64::from(page) >= total_pages,
                        onclick: move |_evt| on_page_change.call(page + 1),
                        "Siguiente"
                    }
                }
            }
        }
    }
}
