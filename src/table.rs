use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Row, Table, Widget},
};
use std::path::Path;

#[derive(Debug)]
pub struct WavHeader<'a> {
    pub bit_depth: u16,
    pub sample_rate: u32,
    pub path: &'a Path,
}

pub struct WavTable<'a> {
    items: &'a [WavHeader<'a>],
    selected: usize,
}

impl<'a> WavTable<'a> {
    pub fn new(items: &'a [WavHeader<'a>], selected: usize) -> Self {
        Self { items, selected }
    }
}

impl Widget for WavTable<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let header = Row::new(vec!["Path", "Sample Rate", "Bit Depth"])
            .style(Style::default().add_modifier(Modifier::BOLD))
            .bottom_margin(1);

        let rows: Vec<Row> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, wav)| {
                let style = if i == self.selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };
                Row::new(vec![
                    wav.path.display().to_string(),
                    wav.sample_rate.to_string(),
                    wav.bit_depth.to_string(),
                ])
                .style(style)
            })
            .collect();

        let widths = [
            ratatui::layout::Constraint::Percentage(60),
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(20),
        ];

        let table = Table::new(rows, widths).header(header);

        Widget::render(table, area, buf);
    }
}
