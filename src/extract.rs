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

fn construct_blocks<I>(phased_records: I, name: String) -> Option<Vec<Blocks>> 
    where I: Iterator<Item = (i64, i64, u32)>
    {
    // first, sort the phased records by phaseset, then by start position
    let mut phased_records: Vec<_> = phased_records.collect();
    phased_records.sort_by_key(|&(start, _, phaseset)| (phaseset, start));
    let mut phased_records = phased_records.into_iter();
    let mut phaseblocks = vec![];
    let (mut start1, mut block_end, mut phaseset1) = match phased_records.next() {
        Some(record) => record,
        None => return None, // Return None if no phased records are found
    };
    
    for (start, end, phaseset) in phased_records {
        if phaseset == phaseset1 {
            block_end = end;
        } else {
            phaseblocks.push(Blocks {
                start: start1,
                end: block_end,
                name: name.clone(),
                empty: false,
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
        empty: false,
    });

    Some(phaseblocks)
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

    // For the name, use the basename and strip the extension. Return the full path if something goes wrong.
    let name = bamp.file_name()
        .and_then(|f| f.to_str())
        .map(|s| {
            let stem = if let Some(stem) = s.strip_suffix(".cram") {
                stem
            } else if let Some(stem) = s.strip_suffix(".bam") {
                stem
            } else {
                s
            };
            stem.to_string()
        })
        .unwrap_or_else(|| bamp.display().to_string());

    let phased_reads_iter = bam
        .rc_records()
        .map(|r| r.expect("Failure parsing Bam file"))
        .filter(|read| read.flags() & (htslib::BAM_FUNMAP | htslib::BAM_FSECONDARY) as u16 == 0)
        .map(|read| (read.pos(), read.reference_end(), get_phaseset(&read)))
        .filter(|(_, _, p)| p.is_some())
        .map(|(start, end, p)| (start, end, p.unwrap()));

    match construct_blocks(phased_reads_iter, name.clone()) {
        Some(blocks) => Ok(blocks),
        None => {
            eprintln!("Warning: No phased records found in BAM file {}", bamp.display());
            Ok(vec![Blocks {
                start: 0,
                end: 0,
                name,
                empty: true,
            }])
        }
    }
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

    // For the name, use the basename and strip the extension. Return the full path if something goes wrong.
    let name = vcff.file_name()
        .and_then(|f| f.to_str())
        .map(|s| {
            let stem = if let Some(stem) = s.strip_suffix(".vcf") {
                stem
            } else if let Some(stem) = s.strip_suffix(".vcf.gz") {
                stem
            } else {
                s
            };
            stem.to_string()
        })
        .unwrap_or_else(|| vcff.display().to_string());

    vcf.fetch(rid, region.start as u64, Some(region.end as u64))
        .expect("Failed fetching region from VCF");

    let phased_variants = vcf.records()
        .map(|record| record.expect("Failed getting record from VCF"))
        .map(|record| {
            (record.pos(), record.end(), record.format(b"PS").integer())
        })
        .filter(|(_, _, record)| record.is_ok())
        .map(|(start, end, record)| {
            (start, end, Some(*record.expect("Failed getting phaseset from VCF").first().expect("Failed getting phaseset from VCF").first().expect("Failed getting phaseset from VCF") as u32))
        }).filter(|(_, _, phaseset)| phaseset.is_some())
        .map(|(start, end, phaseset)| (start, end, phaseset.unwrap()));

    match construct_blocks(phased_variants, name.clone()) {
        Some(blocks) => Ok(blocks),
        None => {
            eprintln!("Warning: No phased records found in VCF file {}", vcff.display());
            Ok(vec![Blocks {
                start: 0,
                end: 0,
                name,
                empty: true,
            }])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct_blocks() {
        let phased_records = vec![(1, 2, 1), (3, 4, 1), (5, 6, 2), (7, 8, 2)];
        let blocks = construct_blocks(phased_records.into_iter(), "test".to_string()).unwrap();
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