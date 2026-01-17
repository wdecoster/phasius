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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_single_sample_single_block() {
        let blocks = vec![vec![Blocks {
            start: 1000,
            end: 2000,
            name: "sample1".to_string(),
            empty: false,
        }]];
        
        let result = summarize(&blocks);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2); // header + 1 sample
        assert_eq!(lines[0], "sample_name\tnum_blocks\tblock_coordinates");
        assert_eq!(lines[1], "sample1\t1\t1000-2000");
    }

    #[test]
    fn test_summarize_single_sample_multiple_blocks() {
        let blocks = vec![vec![
            Blocks {
                start: 1000,
                end: 2000,
                name: "sample1".to_string(),
                empty: false,
            },
            Blocks {
                start: 5000,
                end: 7000,
                name: "sample1".to_string(),
                empty: false,
            },
        ]];
        
        let result = summarize(&blocks);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1], "sample1\t2\t1000-2000;5000-7000");
    }

    #[test]
    fn test_summarize_multiple_samples() {
        let blocks = vec![
            vec![Blocks {
                start: 1000,
                end: 2000,
                name: "sample1".to_string(),
                empty: false,
            }],
            vec![Blocks {
                start: 3000,
                end: 4000,
                name: "sample2".to_string(),
                empty: false,
            }],
        ];
        
        let result = summarize(&blocks);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3); // header + 2 samples
        assert_eq!(lines[1], "sample1\t1\t1000-2000");
        assert_eq!(lines[2], "sample2\t1\t3000-4000");
    }

    #[test]
    fn test_summarize_empty_blocks() {
        let blocks = vec![vec![Blocks {
            start: 0,
            end: 0,
            name: "sample_empty".to_string(),
            empty: true,
        }]];
        
        let result = summarize(&blocks);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1], "sample_empty\t0\t0");
    }

    #[test]
    fn test_summarize_mixed_empty_and_normal() {
        let blocks = vec![
            vec![Blocks {
                start: 1000,
                end: 2000,
                name: "sample1".to_string(),
                empty: false,
            }],
            vec![Blocks {
                start: 0,
                end: 0,
                name: "sample2".to_string(),
                empty: true,
            }],
        ];
        
        let result = summarize(&blocks);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[1], "sample1\t1\t1000-2000");
        assert_eq!(lines[2], "sample2\t0\t0");
    }
}