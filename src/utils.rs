#[derive(Debug)]
pub struct Reg {
    pub chrom: String,
    pub start: u32,
    pub end: u32,
}

/// parse a region string
pub fn process_region(reg: &str) -> Result<Reg, Box<dyn std::error::Error>> {
    let reg = reg.replace(',', "");
    let chrom = reg.split(':').collect::<Vec<&str>>()[0];
    let interval = reg.split(':').collect::<Vec<&str>>()[1];
    let start: u32 = interval.split('-').collect::<Vec<&str>>()[0].parse()?;
    let end: u32 = interval.split('-').collect::<Vec<&str>>()[1].parse()?;
    if end > start {
        Ok(Reg {
            chrom: chrom.to_string(),
            start,
            end,
        })
    } else {
        Err("Invalid region: begin has to be smaller than end.".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_region_basic() {
        let result = process_region("chr1:1000-2000").unwrap();
        assert_eq!(result.chrom, "chr1");
        assert_eq!(result.start, 1000);
        assert_eq!(result.end, 2000);
    }

    #[test]
    fn test_process_region_with_commas() {
        let result = process_region("chr7:152,743,763-156,779,243").unwrap();
        assert_eq!(result.chrom, "chr7");
        assert_eq!(result.start, 152743763);
        assert_eq!(result.end, 156779243);
    }

    #[test]
    fn test_process_region_no_chr_prefix() {
        let result = process_region("1:1000-2000").unwrap();
        assert_eq!(result.chrom, "1");
        assert_eq!(result.start, 1000);
        assert_eq!(result.end, 2000);
    }

    #[test]
    fn test_process_region_invalid_end_before_start() {
        let result = process_region("chr1:2000-1000");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("begin has to be smaller than end"));
    }

    #[test]
    fn test_process_region_equal_start_end() {
        let result = process_region("chr1:1000-1000");
        assert!(result.is_err());
    }

    #[test]
    #[should_panic]
    fn test_process_region_missing_colon() {
        process_region("chr1-1000-2000").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_process_region_invalid_format() {
        process_region("chr1:not-a-number").unwrap();
    }
}