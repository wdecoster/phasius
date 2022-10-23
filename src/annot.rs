use plotly::{
    color::Rgb,
    common::{Line, Mode},
    Scatter,
};
use rust_htslib::tbx::{self, Read};
use std::path::PathBuf;

pub struct Annot {
    pub begin: i64,
    pub end: i64,
    pub name: Option<String>,
}

impl Annot {
    pub fn plot(self) -> Box<plotly::Scatter<i64, i64>> {
        match self.name {
            Some(s) => Scatter::new(vec![self.begin, self.end], vec![-2, -2])
                .mode(Mode::Lines)
                .name(&s)
                .show_legend(false)
                .line(Line::new().color(Rgb::new(128, 128, 128)).width(3.0)),
            None => Scatter::new(vec![self.begin, self.end], vec![-2, -2])
                .mode(Mode::Lines)
                .show_legend(false)
                .line(Line::new().color(Rgb::new(128, 128, 128)).width(3.0)),
        }
    }
}

pub fn parse_bed(
    p: PathBuf,
    region: &crate::utils::Reg,
) -> Result<Vec<Annot>, Box<dyn std::error::Error>> {
    let mut annotation: Vec<Annot> = vec![];
    let mut tbx_reader = tbx::Reader::from_path(&p)?;
    let tid = tbx_reader.tid(&region.chrom)?;
    tbx_reader.fetch(tid, region.start.into(), region.end.into())?;

    // Read through all records in region.
    for record in tbx_reader.records() {
        let record = record?;
        let line = std::str::from_utf8(&record)?;
        let line_split: Vec<&str> = line.split('\t').collect();
        if line_split.len() > 3 {
            annotation.push(Annot {
                begin: line_split[1].parse()?,
                end: line_split[2].parse()?,
                name: Some(line_split[3].to_owned()),
            })
        } else {
            annotation.push(Annot {
                begin: line_split[1].parse()?,
                end: line_split[2].parse()?,
                name: None,
            })
        }
    }
    Ok(annotation)
}
