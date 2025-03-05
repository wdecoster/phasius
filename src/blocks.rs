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
        limits: Option<(u32, u32)>,
    ) -> Box<plotly::Scatter<i64, usize>> {
        // with limits, the start of the plot cannot be less than the lower limit
        // and the end of the plot cannot be greater than the upper limit
        let (start, end) = if let Some((lower, upper)) = limits {
            (self.start.max(lower as i64), self.end.min(upper as i64))
        } else {
            (self.start, self.end)
        };
        match width {
            Some(width) => {
        Scatter::new(vec![start, end], vec![height, height])
            .mode(Mode::Lines)
            .name(&self.name)
            .legend_group(&self.name)
            .show_legend(show_legend)
            .line(Line::new().width(width as f64))
            .marker(Marker::new().color(color))
            },
            None => {
        Scatter::new(vec![start, end], vec![height, height])
            .mode(Mode::Lines)
            .name(&self.name)
            .legend_group(&self.name)
            .show_legend(show_legend)
            .marker(Marker::new().color(color))
            },
        }
    }
}