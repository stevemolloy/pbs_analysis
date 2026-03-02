use charts_rs::MultiChart;
use genpdf::elements::Paragraph;
use genpdf::{Document, SimplePageDecorator, fonts};

mod nodes;
mod pbs_lib;

use crate::nodes::{FILEPATH, Node, Nodes, cost_plots_from_nodes};
use crate::pbs_lib::{add_chart, add_heading, blank_line, main_title};

const DOC_TITLE: &str = "MAX4U PBS Cost Analysis";

fn main() {
    let nodes = Nodes::from_file(FILEPATH).unwrap();

    let level2_nodes: Vec<&Node> = nodes.get_children(1);
    let total_cost: f32 = nodes.get_unit_cost(1).unwrap() / 1e3;
    let mut level2_charts: MultiChart = cost_plots_from_nodes(
        &level2_nodes,
        &format!("Total cost: {:0.1} M.SEK", total_cost),
    );

    let search_term: &str = "Magnet system";
    let mag_system_list: Vec<&Node> = nodes.get_nodes_with_name(search_term);
    if mag_system_list.len() != 1 {
        panic!("More than one node named \"{search_term}\"");
    }
    let mag_system: &Node = mag_system_list[0];
    let mag_system_children: Vec<&Node> = nodes.get_children(mag_system.id);
    let mut mag_system_charts: MultiChart =
        cost_plots_from_nodes(&mag_system_children, "Mag system");

    let font_family = fonts::from_files("/usr/share/fonts/noto", "NotoSans", None)
        .expect("Failed to load font family");

    let mut doc = Document::new(font_family);
    doc.set_title("Demo doc");

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    doc.set_title(DOC_TITLE);

    doc.push(main_title(DOC_TITLE));
    doc.push(blank_line());

    add_heading(&mut doc, "Cost distribution at project level");
    add_chart(&mut doc, &mut level2_charts);
    add_heading(&mut doc, "Cost distribution of the magnet system");
    add_chart(&mut doc, &mut mag_system_charts);

    let element_count_total: f32 = nodes.count_all_children(1);
    doc.push(Paragraph::new(format!(
        "The Product Breakdown Structure currently contains {element_count_total} elements."
    )));
    doc.push(blank_line());

    doc.render_to_file("output.pdf")
        .expect("Failed to write PDF");
}
