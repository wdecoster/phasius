use bam::ext::BamRecordExtensions;
use log::error;
use plotly::common::{Marker, Mode};
use plotly::Scatter;
use rust_htslib::bam::record::Aux;
use rust_htslib::htslib;
use rust_htslib::{bam, bam::Read};
use std::path::PathBuf; // for BAM_F*

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

pub fn blocks_from_bam(
    bamp: &PathBuf,
    threads: usize,
    region: &crate::utils::Reg,
) -> Result<Vec<Blocks>, Box<dyn std::error::Error>> {
    let mut bam = bam::IndexedReader::from_path(&bamp)?;

    let tid = bam
        .header()
        .tid(region.chrom.as_bytes())
        .ok_or("chromosome not found")?;
    bam.fetch((tid, region.start, region.end))?;
    bam.set_threads(threads)?;

    let mut phased_reads_iter = bam
        .rc_records()
        .map(|r| r.expect("Failure parsing Bam file"))
        .filter(|read| read.flags() & (htslib::BAM_FUNMAP | htslib::BAM_FSECONDARY) as u16 == 0)
        .map(|read| (read.pos(), read.reference_end(), get_phaseset(&read)))
        .filter(|(_, _, p)| p.is_some())
        .peekable();
    if phased_reads_iter.peek().is_none() {
        error!("Not a single phased read found in {}!", bamp.display());
        panic!();
    }
    let mut phaseblocks = vec![];
    let (mut start1, mut block_end, mut phaseset1) = phased_reads_iter
        .next()
        .expect("Not a single phased read found!");
    let name = bamp.file_stem().unwrap().to_str().unwrap().to_string();
    for (start, end, phaseset) in phased_reads_iter {
        if phaseset == phaseset1 {
            block_end = end;
        } else {
            phaseblocks.push(Blocks {
                start: start1,
                end: block_end,
                name: name.clone(),
            });
            start1 = start;
            block_end = end;
            phaseset1 = phaseset;
        }
    }
    phaseblocks.push(Blocks {
        start: start1,
        end: block_end,
        name,
    });

    Ok(phaseblocks)
}

fn get_phaseset(record: &bam::Record) -> Option<u32> {
    match record.aux(b"PS") {
        Ok(value) => match value {
            Aux::U8(v) => Some(u32::from(v)),
            Aux::U16(v) => Some(u32::from(v)),
            Aux::U32(v) => Some(v),
            _ => panic!("Unexpected type of Aux {:?}", value),
        },
        Err(_e) => None,
    }
}
