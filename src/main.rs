use charts_rs::MultiChart;
use genpdf::{Document, SimplePageDecorator, fonts};
use std::cmp::Ordering;
use std::collections::HashMap;

mod nodes;
mod pbs_lib;

use crate::nodes::{FILEPATH, Node, Nodes, cost_plots_from_data, cost_plots_from_nodes};
use crate::pbs_lib::{SimpleDoc, get_magnet_block_costs};

const DOC_TITLE: &str = "MAX4U PBS Cost Analysis";
const OUTPUT_FILE: &str = "output.pdf";
const FONT: &str = "NotoSans";
const FONT_DIR: &str = "/usr/share/fonts/noto";

const ROOT_NODE: u32 = 1;
const MAGSYS_NODE_NAME: &str = "Magnet system";
const MAG_SUBSYSTEMS: &[&str] = &["PS", "Cable", "Main coils", "PFS", "PFS PS", "PFS Cable"];

fn main() {
    let pbs = Nodes::from_file(FILEPATH).unwrap();

    let element_count_total: f32 = pbs.count_all_children(ROOT_NODE);
    let total_cost: f32 = pbs.get_unit_cost(1).unwrap() / 1e3;

    let level2_nodes: Vec<&Node> = pbs.get_children(1);
    let mut level2_charts: MultiChart = cost_plots_from_nodes(
        &level2_nodes,
        &format!("Total cost: {:0.1} M.SEK", total_cost),
    );

    let mag_system: &Node = pbs.get_node_with_name(MAGSYS_NODE_NAME);
    let mut mag_system_charts: MultiChart =
        cost_plots_from_nodes(&pbs.get_children(mag_system.id), "");

    let iron_blocks: &Node = pbs.get_node_with_name("Iron blocks");
    let block_costs: HashMap<String, f32> = get_magnet_block_costs(&pbs);

    let mut mag_subsystem_costs: HashMap<String, f32> = HashMap::new();

    for ss in MAG_SUBSYSTEMS.iter() {
        let ns: Vec<&Node> = pbs.get_nodes_with_name(ss);
        let ns_total_cost: f32 = ns
            .iter()
            .map(|n| n.total_cost.expect("No total_cost"))
            .reduce(|n, e| n + e)
            .expect("Could not reduce the PS costs");
        mag_subsystem_costs.insert(ss.to_string(), ns_total_cost);
    }
    let mut mag_ss_pairs: Vec<(String, f32)> = mag_subsystem_costs.clone().into_iter().collect();
    mag_ss_pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    let mag_ss_names: Vec<String> = mag_ss_pairs.iter().map(|(k, _)| k.clone()).collect();
    let mag_ss_costs: Vec<f32> = mag_ss_pairs.iter().map(|(_, v)| *v / 1e3).collect();
    let mut mag_subsystem_charts =
        cost_plots_from_data(mag_ss_names, mag_ss_costs, "Magnet subsystems");

    let font_family = fonts::from_files(FONT_DIR, FONT, None).expect("Failed to load font family");

    let mut doc = Document::new(font_family);
    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    doc.title(DOC_TITLE)
        .paragraph(
            "This is an analysis of the Product Breakdown Structure of the MAX4U project. \
        As a PBS, it only contains information on the costs of components. \
        There is no information on the costs of consumables or labour.",
        )
        .paragraph(&format!(
            "The Product Breakdown Structure currently contains \
            {element_count_total} elements."
        ))
        .heading("Cost distribution at project level")
        .chart(&mut level2_charts)
        .paragraph(&format!(
            "Note that the magnet system dominates the cost, \
            accounting for {percent:0.1}% of the total.",
            percent = 100.0f32
                * mag_system
                    .total_cost
                    .expect("Mag system does not have a total_cost")
                / 1e3
                / total_cost
        ))
        .heading("Cost distribution of the magnet system")
        .chart(&mut mag_system_charts)
        .paragraph(&format!(
            "Within the magnet system it is the iron blocks that dominate. \
            Altogether they account for {percent:0.1}% of the magnet system.",
            percent = 100.0f32
                * iron_blocks
                    .total_cost
                    .expect("Mag system does not have a total_cost")
                / mag_system
                    .total_cost
                    .expect("Iron blocks does not have a total_cost")
        ))
        .heading("Magnet block costs")
        .magnet_block_cost_table(block_costs)
        .pagebreak()
        .heading("Magnet subsystems")
        .paragraph(
            "For the remainder of the magnet systems \
    i.e., those components that are not iron blocks, the cost distribution is \
    as follows.",
        )
        .chart(&mut mag_subsystem_charts)
        .paragraph(&format!(
            "Note that the main coils account for {:0.1}% \
    of the total of these subsystems.",
            100.0 * mag_subsystem_costs["Main coils"] / mag_subsystem_costs.values().sum::<f32>()
        ));

    doc.render_to_file(OUTPUT_FILE)
        .expect("Failed to write PDF");
}
