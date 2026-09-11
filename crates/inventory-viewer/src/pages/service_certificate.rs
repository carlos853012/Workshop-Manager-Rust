use dioxus::prelude::*;
use inventory_common::dto::{AddRepairPartRequest, RepairDetail, RepairPartResponse, UpdateRepairRequest};
use inventory_common::money::format_clp;
use inventory_common::{Product, Repair};
use rust_decimal::Decimal;

use crate::api::ApiError;
use crate::app_state::use_auth;
use crate::components::atoms::button::{Button, ButtonVariant};
use crate::components::atoms::spinner::Spinner;
use crate::components::molecules::card::Card;
use crate::components::molecules::modal::Modal;
use crate::icons::IconName;
use crate::pages::layout::{require_auth, AppShell};
use crate::routes::Route;

#[component]
pub fn ServiceCertificatePage() -> Element {
    if require_auth().is_none() {
        return rsx! {};
    }

    let mut auth = use_auth();
    let navigator = dioxus_router::prelude::use_navigator();
    let mut search_query = use_signal(String::new);
    let all_repairs = use_signal(Vec::<Repair>::new);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);
    let generating = use_signal(|| false);
    let success_msg = use_signal(|| None::<String>);

    let mut show_detail = use_signal(|| false);
    let mut detail_repair_id = use_signal(|| None::<uuid::Uuid>);

    use_effect(move || {
        let client = auth.api_client();
        let mut loading_set = loading;
        let mut error_set = error;
        let mut repairs_set = all_repairs;
        loading_set.set(true);
        error_set.set(None);
        spawn(async move {
            if let Some(api) = client {
                match api.list_repairs(1, 100).await {
                    Ok(page) => {
                        repairs_set.set(page.items);
                    }
                    Err(ApiError::Unauthorized) | Err(ApiError::Forbidden) => {
                        auth.logout();
                        navigator.push(Route::Login {});
                        return;
                    }
                    Err(e) => {
                        error_set.set(Some(e.user_message().to_string()));
                    }
                }
            }
            loading_set.set(false);
        });
    });

    let filtered: Vec<Repair> = {
        let q = search_query.read().to_lowercase();
        let q = q.trim();
        if q.is_empty() {
            all_repairs.read().clone()
        } else {
            all_repairs
                .read()
                .iter()
                .filter(|r| {
                    let name = r.customer_name.as_deref().unwrap_or("").to_lowercase();
                    let plate = r.license_plate.as_deref().unwrap_or("").to_lowercase();
                    let vehicle = r.vehicle.as_deref().unwrap_or("").to_lowercase();
                    let desc = r.description.as_deref().unwrap_or("").to_lowercase();
                    name.contains(q)
                        || plate.contains(q)
                        || vehicle.contains(q)
                        || desc.contains(q)
                })
                .cloned()
                .collect()
        }
    };

    let download_cert = move |(email, name, plate): (Option<String>, Option<String>, Option<String>)| {
        let client = auth.api_client();
        let mut generating_set = generating;
        let mut error_set = error;
        let mut success_set = success_msg;
        generating_set.set(true);
        error_set.set(None);
        success_set.set(None);
        let mut auth = auth;
        spawn(async move {
            if let Some(api) = client {
                match api.download_certificate(email.as_deref(), name.as_deref(), plate.as_deref()).await {
                    Ok(pdf_bytes) => {
                        let label = email.as_deref().or(name.as_deref()).unwrap_or("cliente");
                        let filename = format!("certificado_{}.pdf", label.replace('@', "_at_"));
                        let path = std::path::PathBuf::from(&filename);
                        if let Err(e) = std::fs::write(&path, &pdf_bytes) {
                            error_set.set(Some(format!("Error al guardar: {}", e)));
                        } else {
                            success_set.set(Some(format!("Certificado guardado en {}", path.display())));
                        }
                    }
                    Err(ApiError::Unauthorized) | Err(ApiError::Forbidden) => {
                        auth.logout();
                        navigator.push(Route::Login {});
                    }
                    Err(e) => {
                        error_set.set(Some(e.user_message().to_string()));
                    }
                }
            }
            generating_set.set(false);
        });
    };

    rsx! {
        AppShell { title: "Certificado de Servicios".to_string(), active_route: Route::ServiceCertificatePage {},
            if let Some(err) = error.read().as_ref() {
                div { class: "alert alert-danger mtsm", "{err}" }
            }
            if let Some(msg) = success_msg.read().as_ref() {
                div { class: "alert alert-success mt-sm", "{msg}" }
            }
            div { class: "cert-search-wrapper",
                    input {
                        class: "cert-search-input",
                        r#type: "text",
                        value: "{search_query}",
                        placeholder: "Filtrar por cliente, patente, vehículo o descripción...",
                        oninput: move |evt: Event<FormData>| search_query.set(evt.value()),
                    }
                }
            Card {
                title: "Reparaciones del taller".to_string(),
                

                if *loading.read() {
                    div { class: "empty-state", Spinner {} }
                } else {
                    div { class: "products-table",
                        table { class: "data-table",
                            thead {
                                tr {
                                    th { "Cliente" }
                                    th { "Vehículo" }
                                    th { "Patente" }
                                    th { "Estado" }
                                    th { "Fecha" }
                                    th { class: "col-actions", "" }
                                }
                            }
                            tbody {
                                if filtered.is_empty() {
                                    tr {
                                        td {
                                            colspan: "6",
                                            div { class: "empty-state",
                                                {IconName::Search.render()}
                                                p { "No se encontraron reparaciones" }
                                            }
                                        }
                                    }
                                } else {
                                    for r in filtered.iter() {
                                        tr {
                                            td { "{r.customer_name.as_deref().unwrap_or(\"-\")}" }
                                            td { "{r.vehicle.as_deref().unwrap_or(\"-\")}" }
                                            td {
                                                if let Some(ref plate) = r.license_plate {
                                                    "{plate.to_uppercase()}"
                                                } else {
                                                    "-"
                                                }
                                            }
                                            td {
                                                "{r.status}"
                                            }
                                            td { class: "text-muted",
                                                "{r.created_at.format(\"%d-%m-%Y\").to_string()}"
                                            }
                                            td { class: "col-actions",
                                                div { class: "table-actions",
                                                    button {
                                                        class: "btn-icon btn-edit",
                                                        title: "Ver detalle",
                                                        onclick: {
                                                            let rid = r.id;
                                                            move |e: Event<MouseData>| {
                                                                e.stop_propagation();
                                                                detail_repair_id.set(Some(rid));
                                                                show_detail.set(true);
                                                            }
                                                        },
                                                        {IconName::Eye.render()}
                                                    }
                                                    button {
                                                        class: "btn-icon btn-danger",
                                                        title: "Descargar certificado",
                                                        onclick: {
                                                            let email = r.customer_email.clone();
                                                            let name = r.customer_name.clone();
                                                            let plate = r.license_plate.clone();
                                                            move |e: Event<MouseData>| {
                                                                e.stop_propagation();
                                                                download_cert((email.clone(), name.clone(), plate.clone()));
                                                            }
                                                        },
                                                        disabled: *generating.read(),
                                                        if *generating.read() {
                                                            Spinner {}
                                                        } else {
                                                            {IconName::Download.render()}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(rid) = *detail_repair_id.read() {
                CertificateDetailModal {
                    show: *show_detail.read(),
                    repair_id: rid,
                    on_close: move |_| {
                        show_detail.set(false);
                        detail_repair_id.set(None);
                    },
                    on_download: {
                        let auth = auth;
                        let mut generating_set = generating;
                        let mut error_set = error;
                        let mut success_set = success_msg;
                        move |(email, name, plate): (Option<String>, Option<String>, Option<String>)| {
                            generating_set.set(true);
                            error_set.set(None);
                            success_set.set(None);
                            let client = auth.api_client();
                            let mut auth = auth;
                            spawn(async move {
                                if let Some(api) = client {
                                    match api.download_certificate(email.as_deref(), name.as_deref(), plate.as_deref()).await {
                                        Ok(pdf_bytes) => {
                                            let label = email.as_deref().or(name.as_deref()).unwrap_or("cliente");
                                            let filename = format!("certificado_{}.pdf", label.replace('@', "_at_"));
                                            let path = std::path::PathBuf::from(&filename);
                                            if let Err(e) = std::fs::write(&path, &pdf_bytes) {
                                                error_set.set(Some(format!("Error al guardar: {}", e)));
                                            } else {
                                                success_set.set(Some(format!("Certificado guardado en {}", path.display())));
                                            }
                                        }
                                        Err(ApiError::Unauthorized) | Err(ApiError::Forbidden) => {
                                            auth.logout();
                                        }
                                        Err(e) => {
                                            error_set.set(Some(e.user_message().to_string()));
                                        }
                                    }
                                }
                                generating_set.set(false);
                            });
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn CertificateDetailModal(
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
    let mut labor_cost_input = use_signal(String::new);
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
            let qty: Decimal =
                new_part_qty.read().parse().unwrap_or(Decimal::ONE);
            let cost: Option<Decimal> =
                new_part_cost.read().parse().ok();
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

    let total_parts: Decimal = parts
        .read()
        .iter()
        .filter_map(|p| p.total_cost)
        .sum();
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
            on_close: on_close,
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
                div { class: "alert alert-danger", "{err}" }
            } else if let Some(ref d) = *detail.read() {
                { let rep = &d.repair; rsx! {
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
                    div { class: "detail-section",
                        div { class: "detail-row",
                            span { class: "detail-label", "Cliente" }
                            span { "{rep.customer_name.as_deref().unwrap_or(\"-\")}" }
                        }
                        div { class: "detail-row",
                            span { class: "detail-label", "Email" }
                            span { "{rep.customer_email.as_deref().unwrap_or(\"-\")}" }
                        }
                        div { class: "detail-row",
                            span { class: "detail-label", "Teléfono" }
                            span { "{rep.customer_phone.as_deref().unwrap_or(\"-\")}" }
                        }
                        div { class: "detail-row",
                            span { class: "detail-label", "Vehículo" }
                            span { "{rep.vehicle.as_deref().unwrap_or(\"-\")}" }
                        }
                        div { class: "detail-row",
                            span { class: "detail-label", "Patente" }
                            span {
                                if let Some(ref plate) = rep.license_plate {
                                    "{plate.to_uppercase()}"
                                } else {
                                    "-"
                                }
                            }
                        }
                        div { class: "detail-row",
                            span { class: "detail-label", "Estado" }
                            span { "{rep.status}" }
                        }
                        if let Some(ref desc) = rep.description {
                            if !desc.is_empty() {
                                div { class: "form-group mt-md",
                                    label { class: "form-label", "Descripción" }
                                    p { "{desc}" }
                                }
                            }
                        }
                        if let Some(ref diag) = rep.diagnosis {
                            if !diag.is_empty() {
                                div { class: "form-group mt-md",
                                    label { class: "form-label", "Diagnóstico" }
                                    p { "{diag}" }
                                }
                            }
                        }
                    }
                }
                if *active_tab.read() == "parts" {
                    div { class: "detail-section",
                        label { class: "form-label", "Repuestos / Insumos" }
                        if parts.read().is_empty() {
                            p { class: "text-muted", "Sin insumos registrados" }
                        } else {
                            table { class: "data-table mt-sm",
                                thead {
                                    tr {
                                        th { "Nombre" }
                                        th { "Cant." }
                                        th { "Costo Unit." }
                                        th { "Total" }
                                        th { "" }
                                    }
                                }
                                tbody {
                                    for part in parts.read().iter() {
                                        tr {
                                            td { "{part.name}" }
                                            td { "{part.quantity}" }
                                            td {
                                                { part.unit_cost
                                                    .map(format_clp)
                                                    .unwrap_or_else(|| "-".to_string()) }
                                            }
                                            td {
                                                { part.total_cost
                                                    .map(format_clp)
                                                    .unwrap_or_else(|| "-".to_string()) }
                                            }
                                            td {
                                                button {
                                                    class: "btn-icon btn-danger",
                                                    title: "Eliminar",
                                                    onclick: {
                                                        let part_id = part.id;
                                                        move |_| remove_part(part_id)
                                                    },
                                                    {IconName::Trash.render()}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "form-row mt-md",
                            div { class: "form-group",
                                label { class: "form-label", "Producto (opcional)" }
                                select {
                                    class: "input",
                                    value: {
                                        match selected_product_id.read().as_ref() {
                                            Some(id) => id.to_string(),
                                            None => String::new(),
                                        }
                                    },
                                    onchange: move |evt: Event<FormData>| {
                                        let val = evt.value();
                                        if val.is_empty() {
                                            selected_product_id.set(None);
                                        } else {
                                            if let Ok(id) = uuid::Uuid::parse_str(&val) {
                                                selected_product_id.set(Some(id));
                                                if let Some(prod) = products.read().iter().find(|p| p.id == id) {
                                                    new_part_name.set(prod.name.clone());
                                                    new_part_cost.set(prod.price.to_string());
                                                }
                                            }
                                        }
                                    },
                                    option { value: "", "-- Seleccionar producto --" }
                                    for prod in products.read().iter() {
                                        option { value: "{prod.id}", "{prod.name}" }
                                    }
                                }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "Cantidad" }
                                input {
                                    class: "input",
                                    r#type: "number",
                                    value: new_part_qty.read().clone(),
                                    oninput: move |evt: Event<FormData>| new_part_qty.set(evt.value().clone()),
                                }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "Costo unitario" }
                                input {
                                    class: "input",
                                    r#type: "number",
                                    value: new_part_cost.read().clone(),
                                    oninput: move |evt: Event<FormData>| new_part_cost.set(evt.value().clone()),
                                }
                            }
                            div { class: "form-group",
                                label { class: "form-label", " " }
                                button {
                                    class: "btn-icon btn-primary",
                                    title: "Agregar insumo",
                                    onclick: add_part,
                                    {IconName::Plus.render()}
                                }
                            }
                        }
                        div { class: "cost-summary mt-md",
                            label { class: "form-label", "Costo de mano de obra" }
                            div { class: "form-row",
                                div { class: "form-group",
                                    input {
                                        class: "input",
                                        r#type: "text",
                                        placeholder: "$0",
                                        value: labor_cost_input.read().clone(),
                                        oninput: move |evt: Event<FormData>| labor_cost_input.set(evt.value().clone()),
                                    }
                                }
                                div { class: "form-group",
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        onclick: {
                                            let repair_id = repair_id;
                                            move |_| {
                                                let parsed: Option<Decimal> =
                                                    labor_cost_input.read().replace(".", "").replace(",", "").replace("$", "").replace(" ", "").parse().ok();
                                                let client = auth.api_client();
                                                let req = UpdateRepairRequest {
                                                    status: None,
                                                    diagnosis: None,
                                                    technician_id: None,
                                                    estimated_cost: None,
                                                    final_cost: None,
                                                    labor_cost: parsed,
                                                    estimated_delivery: None,
                                                };
                                                spawn(async move {
                                                    if let Some(client) = client {
                                                        let _ = client.update_repair(repair_id, &req).await;
                                                    }
                                                });
                                            }
                                        },
                                        "Guardar"
                                    }
                                }
                            }
                            div { class: "cost-totals mt-sm",
                                div { class: "cost-row",
                                    span { "Mano de obra" }
                                    span { "{labor_cost_input.read()}" }
                                }
                                div { class: "cost-row",
                                    span { "Repuestos" }
                                    span { "{format_clp(total_parts)}" }
                                }
                                div { class: "cost-row cost-total",
                                    span { strong { "Total" } }
                                    span { strong { "{format_clp(total_repair)}" } }
                                }
                            }
                        }
                    }
                }
                }}
            } else {
                div { class: "empty-state", "No se encontró la reparación" }
            }
        }
    }
}
