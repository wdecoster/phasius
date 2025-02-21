use plotly::common::{Marker, Mode, Line};
use plotly::Scatter;

pub struct Blocks {
    pub start: i64,
    pub end: i64,
    pub name: String,
}

impl Blocks {
    pub fn plot(
        &self,
        height: usize,
        color: String,
        show_legend: bool,
        width: Option<usize>,
    ) -> Box<plotly::Scatter<i64, usize>> {
        match width {
            Some(width) => {
        Scatter::new(vec![self.start, self.end], vec![height, height])
            .mode(Mode::Lines)
            .name(&self.name)
            .legend_group(&self.name)
            .show_legend(show_legend)
            .line(Line::new().width(width as f64))
            .marker(Marker::new().color(color))
            },
            None => {
        Scatter::new(vec![self.start, self.end], vec![height, height])
            .mode(Mode::Lines)
            .name(&self.name)
            .legend_group(&self.name)
            .show_legend(show_legend)
            .marker(Marker::new().color(color))
            },
        }
    }
}