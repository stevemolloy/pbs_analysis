use calamine::{DataType, HeaderRow, Reader, Xlsx, open_workbook};
use charts_rs::{BarChart, Box, ChildChart, MultiChart, PieChart, Series, svg_to_png};
use std::fs::File;
use std::io::prelude::*;

const FILEPATH: &str = "ProductBreakdownStructure.xlsx";
const SHEETNAME: &str = "Full Data";

#[derive(Debug)]
struct Node {
    id: u32,
    parent: Option<u32>,
    name: String,
    unit_cost: Option<f32>,
    count: f32,
    total_cost: Option<f32>,
}

struct Nodes {
    data: Vec<Node>,
}

impl Nodes {
    fn get_node_with_id(&self, id: u32) -> Option<&Node> {
        self.data.iter().find(|node| node.id == id)
    }

    fn get_mut_node_with_id(&mut self, id: u32) -> Option<&mut Node> {
        self.data.iter_mut().find(|node| node.id == id)
    }

    fn get_nodes_with_parent(&self, parent: u32) -> Vec<&Node> {
        let mut retval: Vec<&Node> = Vec::new();
        for node in self.data.iter() {
            if node.parent == Some(parent) {
                retval.push(node);
            }
        }
        retval
    }

    fn set_unit_cost(&mut self, id: u32) -> () {
        if self
            .get_node_with_id(id)
            .and_then(|n| n.unit_cost)
            .is_some()
        {
            return;
        }

        let child_ids: Vec<(u32, f32)> = self
            .data
            .iter()
            .filter(|n| n.parent == Some(id))
            .map(|n| (n.id, n.count))
            .collect();

        let mut cost: f32 = 0.0;
        for (child_id, child_count) in child_ids {
            self.set_unit_cost(child_id);
            cost +=
                self.get_node_with_id(child_id).unwrap().unit_cost.unwrap() * child_count as f32;
        }

        let node = self.get_mut_node_with_id(id).unwrap();
        node.unit_cost = Some(cost);
        node.total_cost = Some(cost * node.count as f32);
    }

    fn get_unit_cost(&self, id: u32) -> Option<f32> {
        if let Some(node) = self.get_node_with_id(id) {
            if let Some(cost) = node.unit_cost {
                Some(cost)
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut excel: Xlsx<_> = open_workbook(FILEPATH).unwrap();

    let mut nodes = Nodes { data: Vec::new() };

    if let Ok(datasheet) = excel
        .with_header_row(HeaderRow::Row(1))
        .worksheet_range(SHEETNAME)
    {
        for row in datasheet.rows() {
            let id: u32 = row[0].get_float().unwrap().round() as u32;
            let parent: Option<u32> = if row[1].is_empty() {
                None
            } else {
                Some(row[1].get_float().unwrap().round() as u32)
            };
            let name: String = row[2].to_string();
            let unit_cost: Option<f32> = if row[3].is_empty() {
                None
            } else {
                Some(row[3].get_float().unwrap() as f32)
            };
            let count: f32 = row[4].get_float().unwrap() as f32;
            let total_cost: Option<f32> = if unit_cost.is_some() {
                Some(unit_cost.unwrap() * count as f32)
            } else {
                None
            };

            let node = Node {
                id: id,
                parent: parent,
                name: name,
                unit_cost: unit_cost,
                count: count,
                total_cost: total_cost,
            };

            nodes.data.push(node);
        }
    } else {
        println!(
            "Couldn't open the sheet '{}' in file '{}'",
            SHEETNAME, FILEPATH
        );
        return Ok(());
    }
    nodes.set_unit_cost(1);

    let total_cost: f32 = nodes.get_unit_cost(1).unwrap() / 1e3;
    let level2_nodes = nodes.get_nodes_with_parent(1);

    let level2_names: Vec<String> = level2_nodes.iter().map(|n| n.name.clone()).collect();
    let level2_costs: Vec<f32> = level2_nodes
        .iter()
        .map(|n| n.total_cost.unwrap() / 1e3)
        .collect();

    let mut barchart = BarChart::new(
        vec![("Level2", level2_costs.clone()).into()],
        level2_names.clone(),
    );
    barchart.width = 800.0;
    barchart.title_text = format!("Total cost: {:0.1} M.SEK", total_cost);
    barchart.title_margin = Some(Box {
        top: 15.0,
        bottom: 5.0,
        ..Default::default()
    });
    barchart.legend_show = Some(false);

    let pie_series = level2_names
        .iter()
        .zip(level2_costs)
        .map(|(name, cost)| Series::new(name.clone(), vec![cost]))
        .collect();

    let mut piechart = PieChart::new(pie_series);
    piechart.rose_type = Some(false);

    let mut container = MultiChart::new();
    container.add(ChildChart::Bar(barchart.clone(), Some((0f32, 0f32))));
    container.add(ChildChart::Pie(
        piechart,
        Some((barchart.x + barchart.width, barchart.y)),
    ));

    let mut output_file = match File::create("plot.png") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("ERROR: Could not open output file");
            return Err(e);
        }
    };
    let svg_contents = container.svg().unwrap();
    let png_contents = svg_to_png(&svg_contents).unwrap();
    let res = output_file.write_all(&png_contents);
    if res.is_err() {
        eprintln!("ERROR: Could not write output file");
        return res;
    }
    Ok(())
}
