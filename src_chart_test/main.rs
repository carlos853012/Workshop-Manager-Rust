fn main() {
    // Replicate what render_line_chart does, then inspect the SVG output
    let data = vec![
        ("2026-04".to_string(), 50000.0_f64),
        ("2026-05".to_string(), 75000.0),
        ("2026-06".to_string(), 60000.0),
        ("2026-07".to_string(), 90000.0),
        ("2026-08".to_string(), 80000.0),
        ("2026-09".to_string(), 95000.0),
    ];
    // Call render_line_chart via workspace dependency
    let svg = workshop_viewer::charts::render_line_chart(&data, plotters::prelude::RGBColor(245, 158, 11), 700, 350).unwrap();
    println!("SVG length: {}", svg.len());
    // Extract and print circle tags
    let mut remaining = &svg[..];
    let mut i = 0;
    while let Some(pos) = remaining.find("<circle") {
        let chunk = &remaining[pos..];
        if let Some(end) = chunk.find("/>") {
            println!("Circle {i}: {}", &chunk[..end+2]);
            i += 1;
            remaining = &remaining[pos+end+2..];
        } else { break; }
    }
    // Extract titles
    for t in svg.split("<title>").skip(1) {
        if let Some(title) = t.split("</title>").next() {
            println!("Title: {title}");
        }
    }
    // Extract text elements
    for t in svg.split("<text").skip(1) {
        if let Some(end) = t.find("</text>") {
            println!("Text: <text{}", &t[..end+7]);
        }
    }
}
