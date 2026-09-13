use genpdf::elements::{FrameCellDecorator, Paragraph, TableLayout};
use genpdf::style::Style;
use genpdf::{Document, Element, SimplePageDecorator};

use crate::routes::reports::ServiceCertificateResponse;

fn load_font_family() -> Result<genpdf::fonts::FontFamily<genpdf::fonts::FontData>, String> {
    let font_dir = std::path::Path::new(r"C:\Windows\Fonts");
    genpdf::fonts::from_files(
        font_dir,
        "LiberationSans",
        Some(genpdf::fonts::Builtin::Helvetica),
    )
    .map_err(|e| format!("Font error: {}", e))
}

/// Generate a PDF certificate for a client's vehicle service history.
pub fn generate_certificate_pdf(cert: &ServiceCertificateResponse) -> Result<Vec<u8>, String> {
    let font_family = load_font_family()?;

    let mut doc = Document::new(font_family);

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(20);
    doc.set_page_decorator(decorator);

    doc.set_title("Certificado de Servicios");

    // Header: Workshop name
    doc.push(Paragraph::new(&cert.workshop.name).styled(Style::new().with_font_size(18).bold()));
    doc.push(
        Paragraph::new(format!("{}, {}", cert.workshop.address, cert.workshop.city))
            .styled(Style::new().with_font_size(10)),
    );

    // Title
    doc.push(
        Paragraph::new("CERTIFICADO DE SERVICIOS").styled(Style::new().with_font_size(16).bold()),
    );

    // Client & Vehicle info table
    let mut info_table = TableLayout::new(vec![1, 2]);
    info_table.set_cell_decorator(FrameCellDecorator::new(false, false, false));

    let client_name = cert.client.name.as_deref().unwrap_or("-");
    let client_email = cert.client.email.as_deref().unwrap_or("-");
    let client_phone = cert.client.phone.as_deref().unwrap_or("-");
    let vehicle_desc = cert.vehicle.description.as_deref().unwrap_or("-");
    let plate = cert.vehicle.license_plate.as_deref().unwrap_or("-");

    add_labeled_row(&mut info_table, "Cliente:", client_name)?;
    add_labeled_row(&mut info_table, "Email:", client_email)?;
    add_labeled_row(&mut info_table, "Telefono:", client_phone)?;
    add_labeled_row(&mut info_table, "Vehiculo:", vehicle_desc)?;
    add_labeled_row(&mut info_table, "Patente:", plate)?;

    doc.push(info_table);

    // Services section
    doc.push(Paragraph::new("SERVICIOS REALIZADOS").styled(Style::new().with_font_size(12).bold()));

    if cert.services.is_empty() {
        doc.push(Paragraph::new("Sin servicios registrados."));
    } else {
        let mut svc_table = TableLayout::new(vec![3, 7]);
        svc_table.set_cell_decorator(FrameCellDecorator::new(true, true, false));

        add_header_row(&mut svc_table, &["Fecha", "Descripcion / Diagnostico"])?;

        for svc in &cert.services {
            let date_str = svc.date.format("%d/%m/%Y").to_string();
            let desc = match (&svc.description, &svc.diagnosis) {
                (Some(d), Some(diag)) => format!("{}\nDiagnostico: {}", d, diag),
                (Some(d), None) => d.clone(),
                (None, Some(diag)) => format!("Diagnostico: {}", diag),
                (None, None) => "-".to_string(),
            };
            add_data_row(&mut svc_table, &[&date_str, &desc])?;
        }

        doc.push(svc_table);
    }

    // Parts section
    doc.push(
        Paragraph::new("REPUESTOS REEMPLAZADOS").styled(Style::new().with_font_size(12).bold()),
    );

    if cert.parts_used.is_empty() {
        doc.push(Paragraph::new("Sin repuestos registrados."));
    } else {
        let mut parts_table = TableLayout::new(vec![4, 1, 5]);
        parts_table.set_cell_decorator(FrameCellDecorator::new(true, true, false));

        add_header_row(&mut parts_table, &["Repuesto", "Cant.", "Observacion"])?;

        for part in &cert.parts_used {
            let observation = match (&part.product_brand, &part.product_model, &part.product_sku) {
                (Some(b), Some(m), Some(s)) => format!("{} {} [{}]", b, m, s),
                (Some(b), Some(m), None) => format!("{} {}", b, m),
                (Some(b), None, Some(s)) => format!("{} [{}]", b, s),
                (Some(b), None, None) => b.clone(),
                (None, Some(m), _) => m.clone(),
                (None, None, Some(s)) => s.clone(),
                (None, None, None) => "-".to_string(),
            };

            let qty_str = part.quantity.to_string();
            add_data_row(&mut parts_table, &[&part.name, &qty_str, &observation])?;
        }

        doc.push(parts_table);
    }

    // Footer
    let gen_date = cert.generated_at.format("%d/%m/%Y").to_string();
    doc.push(
        Paragraph::new(format!("Certificado emitido el {}", gen_date))
            .styled(Style::new().with_font_size(9)),
    );
    doc.push(
        Paragraph::new("Este documento certifica los trabajos realizados en el taller indicado.")
            .styled(Style::new().with_font_size(9)),
    );

    // Render to bytes
    let mut buf = Vec::new();
    doc.render(&mut buf)
        .map_err(|e| format!("PDF render error: {}", e))?;

    Ok(buf)
}

fn add_labeled_row(table: &mut TableLayout, label: &str, value: &str) -> Result<(), String> {
    table
        .row()
        .element(Paragraph::new(label).styled(Style::new().bold()))
        .element(Paragraph::new(value))
        .push()
        .map_err(|e| format!("Table row error: {}", e))
}

fn add_header_row(table: &mut TableLayout, headers: &[&str]) -> Result<(), String> {
    let mut row = table.row();
    for h in headers {
        row = row.element(Paragraph::new(*h).styled(Style::new().bold()));
    }
    row.push().map_err(|e| format!("Table row error: {}", e))
}

fn add_data_row(table: &mut TableLayout, values: &[&str]) -> Result<(), String> {
    let mut row = table.row();
    for v in values {
        row = row.element(Paragraph::new(*v).styled(Style::new().with_font_size(9)));
    }
    row.push().map_err(|e| format!("Table row error: {}", e))
}
