use charts_rs::{CanvasResult, MultiChart, svg_to_jpeg};
use genpdf::elements::{Break, Image, Paragraph, StyledElement};
use genpdf::{Alignment, Document, Element, style};
use std::io::Cursor;

pub trait SvgRenderable {
    fn svg(&mut self) -> CanvasResult<String>;
}

impl SvgRenderable for MultiChart {
    fn svg(&mut self) -> CanvasResult<String> {
        self.svg()
    }
}

pub fn blank_line() -> Break {
    Break::new(1)
}

pub fn main_title(text: &str) -> StyledElement<Paragraph> {
    Paragraph::new(text)
        .aligned(Alignment::Center)
        .styled(style::Style::new().bold().with_font_size(20))
}

pub fn add_heading(document: &mut Document, text: &str) {
    document.push(Paragraph::new(text).styled(style::Style::new().bold().with_font_size(16)));
    document.push(blank_line());
}

pub fn add_chart<T: SvgRenderable>(document: &mut Document, chart: &mut T) {
    let svg_contents = chart.svg().expect("Could not convert plots to SVG");
    let jpg_contents = svg_to_jpeg(&svg_contents).expect("Could not convert SVG plots to JPG");
    let img: Image =
        Image::from_reader(Cursor::new(jpg_contents)).expect("Could not convert image to buffer");
    document.push(img);
    document.push(blank_line());
}
