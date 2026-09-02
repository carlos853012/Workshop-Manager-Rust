use dioxus::prelude::*;

use crate::components::atoms::badge::{Badge, BadgeVariant};
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::molecules::card::Card;
use crate::components::organisms::data_table::{Column, DataTable};
use crate::pages::layout::{require_auth, AppShell};
use inventory_common::Product;

#[component]
pub fn Products() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let products = use_signal(Vec::<Product>::new);
    let page = use_signal(|| 1);
    let total = use_signal(|| 0);

    let columns: Vec<Column<Product>> = vec![
        Column {
            key: "name".to_string(),
            header: "Nombre".to_string(),
            render: |p| rsx! { span { "{p.name}" } },
        },
        Column {
            key: "sku".to_string(),
            header: "SKU".to_string(),
            render: |p| rsx! { span { class: "text-muted", "{p.sku.as_deref().unwrap_or(\"-\")}" } },
        },
        Column {
            key: "price".to_string(),
            header: "Precio".to_string(),
            render: |p| rsx! { span { "${p.price}" } },
        },
        Column {
            key: "stock".to_string(),
            header: "Stock".to_string(),
            render: |p| {
                rsx! {
                    if p.stock <= p.min_stock {
                        Badge { variant: BadgeVariant::Danger, "{p.stock}" }
                    } else {
                        span { "{p.stock}" }
                    }
                }
            },
        },
    ];

    let rows = products.read().clone();

    rsx! {
        AppShell { title: "Productos".to_string(), active_route: "/products".to_string(),
            Card {
                title: "Listado de productos".to_string(),
                footer: rsx! {
                    Button { variant: ButtonVariant::Primary, "Nuevo producto" }
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
