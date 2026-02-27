use charts_rs::{MultiChart, svg_to_jpeg};
use std::fs::File;
use std::io::prelude::*;

mod nodes;

use crate::nodes::{FILEPATH, Node, Nodes, cost_plots_from_nodes};

const OUTPUTFILE: &str = "plot.jpg";

fn main() -> std::io::Result<()> {
    let nodes = Nodes::from_file(FILEPATH).unwrap();

    let level2_nodes: Vec<&Node> = nodes.get_nodes_with_parent(1);
    let total_cost: f32 = nodes.get_unit_cost(1).unwrap() / 1e3;
    let title_text = format!("Total cost: {:0.1} M.SEK", total_cost);
    let mut container: MultiChart = cost_plots_from_nodes(&level2_nodes, &title_text);

    let mut output_file = match File::create(OUTPUTFILE) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("ERROR: Could not open output file");
            return Err(e);
        }
    };
    let svg_contents = container.svg().unwrap();
    let jpg_contents = svg_to_jpeg(&svg_contents).unwrap();
    let res = output_file.write_all(&jpg_contents);
    // let res = output_file.write_all(&jpg_contents.as_bytes());
    if res.is_err() {
        eprintln!("ERROR: Could not write output file");
        return res;
    }
    Ok(())
}
