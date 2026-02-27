use calamine::{DataType, HeaderRow, Reader, Xlsx, open_workbook};
use charts_rs::{BarChart, Box, ChildChart, MultiChart, PieChart, Series};

pub const FILEPATH: &str = "ProductBreakdownStructure.xlsx";
pub const SHEETNAME: &str = "Full Data";

#[derive(Debug)]
pub struct Node {
    id: u32,
    parent: Option<u32>,
    name: String,
    unit_cost: Option<f32>,
    count: f32,
    total_cost: Option<f32>,
}

pub struct Nodes {
    data: Vec<Node>,
}

impl Nodes {
    pub fn from_file(filename: &str) -> Result<Nodes, ()> {
        let mut excel: Xlsx<_> = open_workbook(filename).unwrap();

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
                let total_cost: Option<f32> = unit_cost.map(|ucost| ucost * count);

                let node = Node {
                    id,
                    parent,
                    name,
                    unit_cost,
                    count,
                    total_cost,
                };

                nodes.data.push(node);
            }
        } else {
            println!(
                "Couldn't open the sheet '{}' in file '{}'",
                SHEETNAME, FILEPATH
            );
            return Err(());
        }
        nodes.set_unit_cost(1);
        Ok(nodes)
    }

    fn get_node_with_id(&self, id: u32) -> Option<&Node> {
        self.data.iter().find(|node| node.id == id)
    }

    fn get_mut_node_with_id(&mut self, id: u32) -> Option<&mut Node> {
        self.data.iter_mut().find(|node| node.id == id)
    }

    pub fn get_nodes_with_parent(&self, parent: u32) -> Vec<&Node> {
        let mut retval: Vec<&Node> = Vec::new();
        for node in self.data.iter() {
            if node.parent == Some(parent) {
                retval.push(node);
            }
        }
        retval
    }

    pub fn set_unit_cost(&mut self, id: u32) {
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
            cost += self.get_node_with_id(child_id).unwrap().unit_cost.unwrap() * child_count;
        }

        let node = self.get_mut_node_with_id(id).unwrap();
        node.unit_cost = Some(cost);
        node.total_cost = Some(cost * node.count);
    }

    pub fn get_unit_cost(&self, id: u32) -> Option<f32> {
        if let Some(node) = self.get_node_with_id(id) {
            node.unit_cost
        } else {
            None
        }
    }
}

fn barchart_from_nodes(nodes: &Vec<&Node>) -> BarChart {
    let level2_names: Vec<String> = nodes.iter().map(|n| n.name.clone()).collect();
    let level2_costs: Vec<f32> = nodes.iter().map(|n| n.total_cost.unwrap() / 1e3).collect();

    let mut barchart: BarChart = BarChart::new(
        vec![("Level2", level2_costs.clone()).into()],
        level2_names.clone(),
    );
    barchart.width = 800.0;
    barchart.title_margin = Some(Box {
        top: 15.0,
        bottom: 5.0,
        ..Default::default()
    });
    barchart.legend_show = Some(false);
    barchart
}

fn piechart_from_nodes(nodes: &Vec<&Node>) -> PieChart {
    let level2_names: Vec<String> = nodes.iter().map(|n| n.name.clone()).collect();
    let level2_costs: Vec<f32> = nodes.iter().map(|n| n.total_cost.unwrap() / 1e3).collect();

    let pie_series = level2_names
        .iter()
        .zip(level2_costs)
        .map(|(name, cost)| Series::new(name.clone(), vec![cost]))
        .collect();

    let mut piechart: PieChart = PieChart::new(pie_series);
    piechart.rose_type = Some(false);
    piechart
}

pub fn cost_plots_from_nodes(nodes: &Vec<&Node>, title: &str) -> MultiChart {
    let mut barchart: BarChart = barchart_from_nodes(nodes);
    barchart.title_text = title.to_string();

    let piechart: PieChart = piechart_from_nodes(nodes);

    let mut container = MultiChart::new();
    container.add(ChildChart::Bar(barchart.clone(), Some((0f32, 0f32))));
    container.add(ChildChart::Pie(
        piechart,
        Some((barchart.x + barchart.width, barchart.y)),
    ));
    container.margin.bottom = 0.0;
    container.margin.right = 0.0;
    container
}
