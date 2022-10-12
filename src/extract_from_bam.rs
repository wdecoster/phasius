use bam::ext::BamRecordExtensions;
use rust_htslib::bam::record::Aux;
use rust_htslib::htslib;
use rust_htslib::{bam, bam::Read};
use std::path::PathBuf; // for BAM_F*

pub fn blocks_from_bam(
    bamp: &PathBuf,
    threads: usize,
    region: &crate::utils::Reg,
) -> Result<Vec<(i64, i64)>, Box<dyn std::error::Error>> {
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
        .filter(|(_, _, p)| p.is_some());
    let mut phaseblocks = vec![];
    let (mut start1, mut block_end, mut phaseset1) = phased_reads_iter.next().unwrap();
    for (start, end, phaseset) in phased_reads_iter {
        if phaseset == phaseset1 {
            block_end = end;
        } else {
            phaseblocks.push((start1, block_end));
            start1 = start;
            block_end = end;
            phaseset1 = phaseset;
        }
    }
    phaseblocks.push((start1, block_end));

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
