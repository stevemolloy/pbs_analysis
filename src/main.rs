use charts_rs::MultiChart;
use genpdf::{Document, SimplePageDecorator, fonts};
use std::collections::HashMap;

mod nodes;
mod pbs_lib;

use crate::nodes::{FILEPATH, Node, Nodes, cost_plots_from_nodes};
use crate::pbs_lib::{
    add_chart, add_heading, add_magnet_block_cost_table, add_paragraph, get_magnet_block_costs,
    set_title,
};

const DOC_TITLE: &str = "MAX4U PBS Cost Analysis";

fn main() {
    let nodes = Nodes::from_file(FILEPATH).unwrap();

    let element_count_total: f32 = nodes.count_all_children(1);
    let total_cost: f32 = nodes.get_unit_cost(1).unwrap() / 1e3;

    let level2_nodes: Vec<&Node> = nodes.get_children(1);
    let mut level2_charts: MultiChart = cost_plots_from_nodes(
        &level2_nodes,
        &format!("Total cost: {:0.1} M.SEK", total_cost),
    );

    let search_term: &str = "Magnet system";
    let mag_system: &Node = nodes.get_node_with_name(search_term);
    let mut mag_system_charts: MultiChart =
        cost_plots_from_nodes(&nodes.get_children(mag_system.id), "Mag system");

    let iron_blocks: &Node = nodes.get_node_with_name("Iron blocks");
    let block_costs: HashMap<String, f32> = get_magnet_block_costs(&nodes);

    let font_family = fonts::from_files("/usr/share/fonts/noto", "NotoSans", None)
        .expect("Failed to load font family");

    let mut doc = Document::new(font_family);
    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    set_title(&mut doc, DOC_TITLE);
    add_paragraph(
        &mut doc,
        "This is an analysis of the Product Breakdown Structure of the MAX4U project. \
        As a PBS, it only contains information on the costs of components. \
        There is no information on the costs of consumables or labour.",
    );
    add_paragraph(
        &mut doc,
        &format!(
            "The Product Breakdown Structure currently contains {element_count_total} elements."
        ),
    );
    add_heading(&mut doc, "Cost distribution at project level");
    add_chart(&mut doc, &mut level2_charts);
    add_paragraph(
        &mut doc,
        &format!(
            "Note that the magnet system dominates the cost, accounting for {percent:0.1}% of the total.",
            percent = 100.0f32
                * mag_system
                    .total_cost
                    .expect("Mag system does not have a total_cost")
                / 1e3
                / total_cost
        ),
    );
    add_heading(&mut doc, "Cost distribution of the magnet system");
    add_chart(&mut doc, &mut mag_system_charts);
    add_paragraph(
        &mut doc,
        &format!(
            "Within the magnet system it is the iron blocks that dominate. Altogether they account for {percent:0.1}% of the magnet system.",
            percent = 100.0f32
                * iron_blocks
                    .total_cost
                    .expect("Mag system does not have a total_cost")
                / mag_system
                    .total_cost
                    .expect("Iron blocks does not have a total_cost")
        ),
    );

    add_heading(&mut doc, "Magnet block costs");

    add_magnet_block_cost_table(&mut doc, block_costs);

    doc.render_to_file("output.pdf")
        .expect("Failed to write PDF");
}
