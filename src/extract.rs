use bam::ext::BamRecordExtensions;
use rust_htslib::bam::record::Aux;
use rust_htslib::htslib;
use rust_htslib::{bam, bam::Read};
use rust_htslib::{bcf::IndexedReader, bcf::Read as VcfRead};
use std::path::PathBuf; // for BAM_F*
use crate::blocks::Blocks;

pub fn get_blocks(
    file: &PathBuf,
    threads: usize,
    region: &crate::utils::Reg,
) -> Result<Vec<Blocks>, Box<dyn std::error::Error>> {
    let filename = file.clone().into_os_string().into_string().expect("Failed parsing filename");
    if file.extension().expect("Failed getting file extension") == "cram" || file.extension().expect("Failed getting file extension") == "bam" {
        blocks_from_bam(file, threads, region)
    } else if filename.ends_with("vcf") || filename.ends_with("vcf.gz") {
        blocks_from_vcf(file, region)
    } else {
        panic!("Unsupported file format or file extension not recognized: {}", filename);
    }

}

fn construct_blocks<I>(mut phased_records: I, name: String) -> Vec<Blocks> 
    where I: Iterator<Item = (i64, i64, Option<u32>)>
    {
    let mut phaseblocks = vec![];
    let (mut start1, mut block_end, mut phaseset1) = phased_records
        .next()
        .expect(&format!("Not a single phased record found in the interval for {}!", name));
    for (start, end, phaseset) in phased_records {
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

    phaseblocks
}


fn blocks_from_bam(
    bamp: &PathBuf,
    threads: usize,
    region: &crate::utils::Reg,
) -> Result<Vec<Blocks>, Box<dyn std::error::Error>> {
    let mut bam = bam::IndexedReader::from_path(bamp)?;

    let tid = bam
        .header()
        .tid(region.chrom.as_bytes())
        .ok_or("chromosome not found")?;
    bam.fetch((tid, region.start, region.end))?;
    bam.set_threads(threads)?;

    let phased_reads_iter = bam
        .rc_records()
        .map(|r| r.expect("Failure parsing Bam file"))
        .filter(|read| read.flags() & (htslib::BAM_FUNMAP | htslib::BAM_FSECONDARY) as u16 == 0)
        .map(|read| (read.pos(), read.reference_end(), get_phaseset(&read)))
        .filter(|(_, _, p)| p.is_some());

    Ok(construct_blocks(phased_reads_iter, bamp.display().to_string()))
}

fn get_phaseset(record: &bam::Record) -> Option<u32> {
    match record.aux(b"PS") {
        Ok(value) => match value {
            Aux::U8(v) => Some(u32::from(v)),
            Aux::U16(v) => Some(u32::from(v)),
            Aux::U32(v) => Some(v),
            Aux::I32(v) => Some(v as u32),
            _ => panic!("Unexpected type of Aux {:?}", value),
        },
        Err(_e) => None,
    }
}

fn blocks_from_vcf(
    vcff: &PathBuf,
    region: &crate::utils::Reg,
) -> Result<Vec<Blocks>, Box<dyn std::error::Error>> {
    let mut vcf = IndexedReader::from_path(vcff)?;
    let rid = vcf.header().name2rid(region.chrom.as_bytes()).expect("Failed getting rid");

    vcf.fetch(rid, region.start as u64, Some(region.end as u64))
        .expect("Failed fetching region from VCF");

    let phased_variants = vcf.records().map(|record| record.expect("Failed getting record from VCF")).map(|record| {
        (record.pos(), record.end(), record.format(b"PS").integer())
    }).filter(|(_, _, record)| record.is_ok()).map(|(start, end, record)| {
        (start, end, Some(*record.expect("Failed getting phaseset from VCF").first().expect("Failed getting phaseset from VCF").first().expect("Failed getting phaseset from VCF") as u32))
    });


    Ok(construct_blocks(phased_variants, vcff.display().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct_blocks() {
        let phased_records = vec![(1, 2, Some(1)), (3, 4, Some(1)), (5, 6, Some(2)), (7, 8, Some(2))];
        let blocks = construct_blocks(phased_records.into_iter(), "test".to_string());
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].start, 1);
        assert_eq!(blocks[0].end, 4);
        assert_eq!(blocks[1].start, 5);
        assert_eq!(blocks[1].end, 8);
    }

    #[test]
    fn test_extension() {
        let path = PathBuf::from("test.vcf.gz");
        let filename = path.clone().into_os_string().into_string().expect("Failed parsing filename");
        assert!(filename.ends_with("vcf.gz"));
    }

}