use charts_rs::{MultiChart, svg_to_jpeg};
use genpdf::elements::{Image, Paragraph};
use genpdf::{Alignment, Element, SimplePageDecorator, fonts, style};
use std::io::Cursor;

mod nodes;

use crate::nodes::{FILEPATH, Node, Nodes, cost_plots_from_nodes};

const DOC_TITLE: &str = "MAX4U PBS Cost Analysis";

fn main() {
    let nodes = Nodes::from_file(FILEPATH).unwrap();

    let level2_nodes: Vec<&Node> = nodes.get_children(1);
    let total_cost: f32 = nodes.get_unit_cost(1).unwrap() / 1e3;
    let mut container: MultiChart = cost_plots_from_nodes(
        &level2_nodes,
        &format!("Total cost: {:0.1} M.SEK", total_cost),
    );

    let element_count_total: f32 = nodes.count_all_children(1);
    println!("Total elements = {element_count_total}");
    for subnode in level2_nodes.iter() {
        let element_count_total: f32 = nodes.count_all_children(subnode.id);
        println!("{name}: {count} elements", name=subnode.name, count=element_count_total);
    }

    println!("");
    let search_term: &str = "D1";
    let mag_system_list: Vec<&Node> = nodes.get_nodes_with_name(search_term);
    if mag_system_list.len() != 1 {
        panic!("More than one node named \"{search_term}\"");
    }
    let mag_system: &Node = mag_system_list[0];
    let mag_system_children: Vec<&Node> = nodes.get_children(mag_system.id);
    println!("Total elements = {count}", count=nodes.count_all_children(mag_system.id));
    for subnode in mag_system_children.iter() {
        let element_count_total: f32 = nodes.count_all_children(subnode.id);
        println!("{name}: {count} elements", name=subnode.name, count=element_count_total);
    }

    let svg_contents = container.svg().expect("Could not convert plots to SVG");
    let jpg_contents = svg_to_jpeg(&svg_contents).expect("Could not convert SVG plots to JPG");

    let font_family = fonts::from_files("/usr/share/fonts/noto", "NotoSans", None)
        .expect("Failed to load font family");

    let mut doc = genpdf::Document::new(font_family);
    doc.set_title("Demo doc");

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    doc.set_title(DOC_TITLE);

    let para = Paragraph::new(DOC_TITLE)
        .aligned(Alignment::Center)
        .styled(style::Style::new().bold().with_font_size(20));
    doc.push(para);

    doc.push(Paragraph::new(format!(
        "The Product Breakdown Structure currently contains {element_count_total} elements."
    )));

    let image =
        Image::from_reader(Cursor::new(jpg_contents)).expect("Could not convert image to buffer");
    doc.push(image);

    doc.render_to_file("output.pdf")
        .expect("Failed to write PDF");
}
