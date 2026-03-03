use charts_rs::{CanvasResult, MultiChart, svg_to_jpeg};
use genpdf::elements::{Break, FrameCellDecorator, Image, Paragraph, TableLayout};
use genpdf::style::Style;
use genpdf::{Alignment, Document, Element, Scale, style};
use itertools::Itertools;
use std::collections::HashMap;
use std::io::Cursor;

use crate::nodes::{Node, Nodes};

const TITLE_FONTSIZE: u8 = 16;
const HEADING_FONTSIZE: u8 = 12;
const DEFAULT_FONTSIZE: u8 = 8;

pub trait SvgRenderable {
    fn svg(&mut self) -> CanvasResult<String>;
}

impl SvgRenderable for MultiChart {
    fn svg(&mut self) -> CanvasResult<String> {
        self.svg()
    }
}

pub fn set_title(document: &mut Document, text: &str) {
    document.set_title(text);
    document.push(
        Paragraph::new(text)
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(TITLE_FONTSIZE)),
    );
    document.push(Break::new(1));
}

pub fn add_heading(document: &mut Document, text: &str) {
    document.push(
        Paragraph::new(text).styled(style::Style::new().bold().with_font_size(HEADING_FONTSIZE)),
    );
    document.push(Break::new(1));
}

pub fn add_paragraph(document: &mut Document, text: &str) {
    document
        .push(Paragraph::new(text).styled(style::Style::new().with_font_size(DEFAULT_FONTSIZE)));
    document.push(Break::new(1));
}

pub fn add_chart<T: SvgRenderable>(document: &mut Document, chart: &mut T) {
    let svg_contents = chart.svg().expect("Could not convert plots to SVG");
    let jpg_contents = svg_to_jpeg(&svg_contents).expect("Could not convert SVG plots to JPG");
    let img: Image =
        Image::from_reader(Cursor::new(jpg_contents)).expect("Could not convert image to buffer");
    document.push(
        img.with_alignment(Alignment::Center)
            .with_scale(Scale::new(1.5, 1.5)),
    );
    document.push(Break::new(1));
}

pub fn add_magnet_block_cost_table(document: &mut Document, block_costs: HashMap<String, f32>) {
    let mut block_cost_table = TableLayout::new(vec![1, 2]);
    block_cost_table.set_cell_decorator(FrameCellDecorator::new(true, true, false));
    let mut row = block_cost_table.row();
    row.push_element(
        Paragraph::new("Block")
            .styled(Style::new().bold().with_font_size(DEFAULT_FONTSIZE))
            .padded(1),
    );
    row.push_element(
        Paragraph::new("Cost (M.SEK)")
            .aligned(Alignment::Center)
            .styled(Style::new().bold().with_font_size(DEFAULT_FONTSIZE))
            .padded(1),
    );
    row.push().expect("Table row is invalid");

    let mut total_cost: f32 = 0.0;
    for name in block_costs.clone().into_keys().sorted() {
        let cost = block_costs[&name];
        if cost == 0.0 {
            continue;
        }
        total_cost += cost;
        let mut row = block_cost_table.row();
        row.push_element(
            Paragraph::new(name)
                .styled(Style::new().with_font_size(DEFAULT_FONTSIZE))
                .padded(1),
        );
        row.push_element(
            Paragraph::new(format!("{:0.3}", cost / 1e3))
                .aligned(Alignment::Center)
                .styled(Style::new().with_font_size(DEFAULT_FONTSIZE))
                .padded(1),
        );
        row.push().expect("Table row is invalid");
    }
    let mut row = block_cost_table.row();
    row.push_element(
        Paragraph::new("TOTAL")
            .styled(Style::new().bold().with_font_size(DEFAULT_FONTSIZE))
            .padded(1),
    );
    row.push_element(
        Paragraph::new(format!("{:0.3}", total_cost / 1e3))
            .aligned(Alignment::Center)
            .styled(Style::new().bold().with_font_size(DEFAULT_FONTSIZE))
            .padded(1),
    );
    row.push().expect("Table row is invalid");

    let mut outer_table = TableLayout::new(vec![1, 1, 1]);
    let mut outer_row = outer_table.row();
    outer_row.push_element(Paragraph::new("").padded(1));
    outer_row.push_element(block_cost_table.padded(1));
    outer_row.push_element(Paragraph::new("").padded(1));
    outer_row.push().expect("Table row is invalid");

    document.push(outer_table);
}

pub fn get_magnet_block_costs(pbs: &Nodes) -> HashMap<String, f32> {
    let iron_blocks_node: &Node = pbs.get_node_with_name("Iron blocks");
    let blocks_nodes: Vec<&Node> = pbs.get_children(iron_blocks_node.id);

    let mut node_map: HashMap<String, f32> = HashMap::new();

    for node in blocks_nodes.iter() {
        node_map.insert(
            node.name.clone(),
            node.unit_cost
                .unwrap_or_else(|| panic!("Node: {} has no unit_cost", node.name)),
        );
    }

    node_map
}
