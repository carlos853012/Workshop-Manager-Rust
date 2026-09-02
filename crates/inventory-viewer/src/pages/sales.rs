use dioxus::prelude::*;
use inventory_common::Sale;

use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::molecules::card::Card;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::pages::layout::{require_auth, AppShell};

#[component]
pub fn Sales() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let sales = use_signal(Vec::<Sale>::new);
    let page = use_signal(|| 1);
    let total = use_signal(|| 0);

    let columns: Vec<Column<Sale>> = vec![
        Column {
            key: "customer".to_string(),
            header: "Cliente".to_string(),
            render: |s| rsx! { span { "{s.customer_name.as_deref().unwrap_or(\"-\")}" } },
        },
        Column {
            key: "payment".to_string(),
            header: "Pago".to_string(),
            render: |s| rsx! { span { "{s.payment_method}" } },
        },
        Column {
            key: "total".to_string(),
            header: "Total".to_string(),
            render: |s| rsx! { span { "${s.total}" } },
        },
        Column {
            key: "status".to_string(),
            header: "Estado".to_string(),
            render: |s| rsx! { span { class: "badge", "{s.status}" } },
        },
    ];

    let rows = sales.read().clone();

    rsx! {
        AppShell { title: "Ventas".to_string(), active_route: "/sales".to_string(),
            Card {
                title: "Listado de ventas".to_string(),
                footer: rsx! {
                    Button { variant: ButtonVariant::Primary, "Nueva venta" }
                },
                DataTable {
                    columns: columns.clone(),
                    rows: rows.clone(),
                    page: *page.read(),
                    total: *total.read(),
                }
            }
        }
    }
}
