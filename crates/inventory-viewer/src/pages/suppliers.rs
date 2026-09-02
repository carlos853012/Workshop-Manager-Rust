use dioxus::prelude::*;
use inventory_common::Supplier;

use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::molecules::card::Card;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::pages::layout::{require_auth, AppShell};

#[component]
pub fn Suppliers() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let suppliers = use_signal(Vec::<Supplier>::new);
    let page = use_signal(|| 1);
    let total = use_signal(|| 0);

    let columns: Vec<Column<Supplier>> = vec![
        Column {
            key: "name".to_string(),
            header: "Nombre".to_string(),
            render: |s| rsx! { span { "{s.name}" } },
        },
        Column {
            key: "contact".to_string(),
            header: "Contacto".to_string(),
            render: |s| rsx! { span { "{s.contact_person.as_deref().unwrap_or(\"-\")}" } },
        },
        Column {
            key: "email".to_string(),
            header: "Email".to_string(),
            render: |s| rsx! { span { "{s.email.as_deref().unwrap_or(\"-\")}" } },
        },
        Column {
            key: "phone".to_string(),
            header: "Teléfono".to_string(),
            render: |s| rsx! { span { "{s.phone.as_deref().unwrap_or(\"-\")}" } },
        },
    ];

    let rows = suppliers.read().clone();

    rsx! {
        AppShell { title: "Proveedores".to_string(), active_route: "/suppliers".to_string(),
            Card {
                title: "Listado de proveedores".to_string(),
                footer: rsx! {
                    Button { variant: ButtonVariant::Primary, "Nuevo proveedor" }
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
