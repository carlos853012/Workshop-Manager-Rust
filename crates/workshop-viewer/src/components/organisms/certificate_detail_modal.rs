use dioxus::prelude::*;
use rust_decimal::Decimal;
use workshop_common::dto::{AddRepairPartRequest, RepairDetail, RepairPartResponse};
use workshop_common::money::format_clp;
use workshop_common::Product;

use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::modal::Modal;
use crate::components::organisms::tabs::info_tab::InfoTab;
use crate::components::organisms::tabs::parts_tab_cert::PartsTabCert;
use crate::icons::IconName;

#[component]
pub fn CertificateDetailModal(
    show: bool,
    repair_id: uuid::Uuid,
    on_close: EventHandler<()>,
    on_download: EventHandler<(Option<String>, Option<String>, Option<String>)>,
) -> Element {
    let auth = use_auth();
    let loading = use_signal(|| true);
    let error = use_signal(|| None::<String>);
    let detail = use_signal(|| None::<RepairDetail>);
    let mut parts = use_signal(Vec::<RepairPartResponse>::new);
    let products = use_signal(Vec::<Product>::new);
    let mut selected_product_id = use_signal(|| None::<uuid::Uuid>);
    let mut new_part_name = use_signal(String::new);
    let mut new_part_qty = use_signal(|| "1".to_string());
    let mut new_part_cost = use_signal(String::new);
    let labor_cost_input = use_signal(String::new);
    let mut active_tab = use_signal(|| "info".to_string());

    use_effect(move || {
        let client = auth.api_client();
        let mut loading_set = loading;
        let mut error_set = error;
        let mut detail_set = detail;
        let mut parts_set = parts;
        let mut products_set = products;
        let mut labor_cost_set = labor_cost_input;
        loading_set.set(true);
        error_set.set(None);
        spawn(async move {
            if let Some(client) = client {
                match client.get_repair(repair_id).await {
                    Ok(d) => {
                        if let Some(ref lc) = d.repair.labor_cost {
                            labor_cost_set.set(format_clp(*lc));
                        }
                        detail_set.set(Some(d));
                    }
                    Err(e) => {
                        error_set.set(Some(e.user_message().to_string()));
                    }
                }
                if let Ok(p) = client.list_repair_parts(repair_id).await {
                    parts_set.set(p);
                }
                if let Ok(prod_page) = client.list_products(1, 100).await {
                    products_set.set(prod_page.items);
                }
            }
            loading_set.set(false);
        });
    });

    let add_part = {
        let repair_id = repair_id;
        move |_| {
            let name = new_part_name.read().trim().to_string();
            if name.is_empty() {
                return;
            }
            let qty: Decimal = new_part_qty.read().parse().unwrap_or(Decimal::ONE);
            let cost: Option<Decimal> = new_part_cost.read().parse().ok();
            let product_id = *selected_product_id.read();
            let client = auth.api_client();
            spawn(async move {
                if let Some(client) = client {
                    let req = AddRepairPartRequest {
                        name,
                        quantity: qty,
                        unit_cost: cost,
                        product_id,
                    };
                    if let Ok(part) = client.add_repair_part(repair_id, &req).await {
                        let mut current = parts.read().clone();
                        current.push(part);
                        parts.set(current);
                        new_part_name.set(String::new());
                        new_part_qty.set("1".to_string());
                        new_part_cost.set(String::new());
                        selected_product_id.set(None);
                    }
                }
            });
        }
    };

    let remove_part = {
        let repair_id = repair_id;
        move |part_id: uuid::Uuid| {
            let client = auth.api_client();
            spawn(async move {
                if let Some(client) = client {
                    if client.remove_repair_part(repair_id, part_id).await.is_ok() {
                        let current = parts.read().clone();
                        let filtered: Vec<RepairPartResponse> =
                            current.into_iter().filter(|x| x.id != part_id).collect();
                        parts.set(filtered);
                    }
                }
            });
        }
    };

    let total_parts: Decimal = parts.read().iter().filter_map(|p| p.total_cost).sum();
    let labor_val: Decimal = labor_cost_input
        .read()
        .replace(".", "")
        .replace(",", "")
        .replace("$", "")
        .replace(" ", "")
        .parse()
        .unwrap_or_default();
    let total_repair = labor_val + total_parts;

    rsx! {
        Modal {
            show: show,
            title: "Detalle de Reparación".to_string(),
            on_close: move |_| on_close.call(()),
            class: Some("modal-lg".to_string()),
            footer: rsx! {
                Button {
                    class: Some("cancel-button".to_string()),
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| on_close.call(()),
                    "Cancelar"
                }
                if let Some(ref d) = *detail.read() {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: {
                            let email = d.repair.customer_email.clone();
                            let name = d.repair.customer_name.clone();
                            let plate = d.repair.license_plate.clone();
                            move |_| on_download.call((email.clone(), name.clone(), plate.clone()))
                        },
                        {IconName::DocumentText.render()}
                        " Descargar certificado"
                    }
                }
            },
            if *loading.read() {
                div { class: "empty-state", Spinner {} }
            } else if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mb-md", "{err}" }
            } else if let Some(ref d) = *detail.read() {
                { let d = d.clone(); rsx! {
                    div { class: "tabs mb-md",
                        button {
                            class: if *active_tab.read() == "info" { "tab-link active" } else { "tab-link" },
                            onclick: move |_| active_tab.set("info".to_string()),
                            "Información"
                        }
                        button {
                            class: if *active_tab.read() == "parts" { "tab-link active" } else { "tab-link" },
                            onclick: move |_| active_tab.set("parts".to_string()),
                            "Insumos"
                        }
                    }
                    if *active_tab.read() == "info" {
                        InfoTab { detail: d.clone() }
                    }
                    if *active_tab.read() == "parts" {
                        PartsTabCert {
                            parts: parts,
                            products: products,
                            selected_product_id: selected_product_id,
                            new_part_name: new_part_name,
                            new_part_qty: new_part_qty,
                            new_part_cost: new_part_cost,
                            labor_cost_input: labor_cost_input,
                            total_parts: total_parts,
                            total_repair: total_repair,
                            add_part: add_part,
                            remove_part: remove_part,
                            repair_id: repair_id,
                        }
                    }
                }}
            } else {
                div { class: "empty-state", "No se encontró la reparación" }
            }
        }
    }
}
