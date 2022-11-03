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
    if end - start > 0 {
        Ok(Reg {
            chrom: chrom.to_string(),
            start,
            end,
        })
    } else {
        Err("Invalid region: begin has to be smaller than end.".into())
    }
}
