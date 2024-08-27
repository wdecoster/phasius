use plotly::common::{Marker, Mode};
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
    ) -> Box<plotly::Scatter<i64, usize>> {
        Scatter::new(vec![self.start, self.end], vec![height, height])
            .mode(Mode::Lines)
            .name(&self.name)
            .legend_group(&self.name)
            .show_legend(show_legend)
            .marker(Marker::new().color(color))
    }
}