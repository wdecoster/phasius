use crate::blocks::Blocks;

// for each sample, write one line per block specifying its name, the number of blocks, and a list of blocks
// with their start and end positions
// the end result is a tab-separated file with the following format:
// Sample_name\tnum_blocks\tstart1-end1;start2-end2;...;startN-endN\n
pub fn summarize(blocks: &[Vec<Blocks>]) -> String {
    let mut summary = String::new();
    
    // Add header line
    summary.push_str("sample_name\tnum_blocks\tblock_coordinates\n");
    
    for blocks in blocks.iter() {
        let name = String::from(&blocks[0].name);
        if blocks[0].empty {
            summary.push_str(&format!("{}\t0\t0\n", name));
            continue;
        }
        let num_blocks = blocks.len();
        let blocks = blocks
            .iter()
            .map(|block| format!("{}-{}", block.start, block.end))
            .collect::<Vec<String>>()
            .join(";");
        summary.push_str(&format!("{}\t{}\t{}\n", name, num_blocks, blocks));
    }
    summary
}
