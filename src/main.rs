use clap::Parser;
use log::info;
use plotly::layout::{Axis, Legend};
use plotly::{Layout, Plot};
use rayon::prelude::*;
use std::path::PathBuf;

pub mod annot;
pub mod blocks;
pub mod extract;
pub mod summary;
pub mod utils;

fn validate_file_exists(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if path.exists() && path.is_file() {
        Ok(path)
    } else {
        Err(format!(
            "File '{}' does not exist or is not a file",
            path.display()
        ))
    }
}

// The arguments end up in the Cli struct
#[derive(Parser, Debug)]
#[command(author, version, about="Tool to draw a map of phaseblocks across crams/bams", long_about = None)]
struct Cli {
    /// cram or bam files to check
    #[arg(required = true, value_parser = validate_file_exists, value_name = "FILE")]
    input: Vec<PathBuf>,

    /// bed file annotation to use (bgzipped and tabix indexed)
    #[arg(short, long, value_parser = validate_file_exists)]
    bed: Option<PathBuf>,

    /// Number of crams/bams to parse in parallel
    #[arg(short, long, default_value_t = 4)]
    threads: usize,

    /// Number of decompression threads to use per cram/bam
    #[arg(short, long, default_value_t = 1)]
    decompression: usize,

    /// HTML output file name
    #[arg(short, long)]
    output: String,

    /// region string to plot phase blocks from
    #[arg(short, long)]
    region: String,

    /// line width
    #[arg(short, long)]
    width: Option<usize>,

    /// summary file
    #[arg(long)]
    summary: Option<String>,

    /// strictly plot the begin and end of the specified interval, not the whole interval gathered from blocks
    #[arg(long)]
    strict: bool,
}

fn main() {
    env_logger::init();
    log::debug!("Starting phasius");
    let args = Cli::parse();
    log::debug!("Parsed command line arguments: {:?}", args);
    info!("Collected arguments");
    run_phasius(args);
    log::debug!("phasius completed successfully");
}

fn run_phasius(args: Cli) {
    log::debug!("Starting run_phasius with region: {}", args.region);
    let target = utils::process_region(&args.region).expect("Error: Improper interval!");
    log::debug!("Parsed region: {:?}", target);
    log::debug!(
        "Extracting blocks from {} files with {} threads",
        args.input.len(),
        args.threads
    );
    let blocks_per_bam = extract_blocks(&args, &target);
    log::debug!("Extracted blocks from all files");
    log::debug!("Starting plot generation");
    plot_blocks(&blocks_per_bam, &args, target);
    log::debug!("Plot generation complete");
    if let Some(summary) = args.summary {
        log::debug!("Generating summary file: {}", summary);
        let summary_per_sample = summary::summarize(&blocks_per_bam);
        // write the summary_per_sample to a file
        std::fs::write(&summary, summary_per_sample).expect("Unable to write file");
        log::debug!("Summary file written: {}", summary);
    }
    log::debug!("run_phasius completed");
}

fn extract_blocks(args: &Cli, target: &utils::Reg) -> Vec<Vec<blocks::Blocks>> {
    log::debug!(
        "Extracting blocks from {} files with {} threads",
        args.input.len(),
        args.threads
    );
    let input = args.input.clone();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build()
        .unwrap();

    pool.install(|| {
        input
            .into_par_iter()
            .map(|b| {
                extract::get_blocks(&b, args.decompression, target)
                    .expect("Failure when parsing region from bam file.")
            })
            .collect()
    })
}

fn plot_blocks(blocks_per_bam: &[Vec<blocks::Blocks>], args: &Cli, target: utils::Reg) {
    log::debug!("Plotting {} samples", blocks_per_bam.len());
    let mut plot = Plot::new();
    let default_colors = [
        "#1f77b4", // muted blue
        "#ff7f0e", // safety orange
        "#2ca02c", // cooked asparagus green
        "#d62728", // brick red
        "#9467bd", // muted purple
        "#8c564b", // chestnut brown
        "#e377c2", // raspberry yogurt pink
        "#7f7f7f", // middle gray
        "#bcbd22", // curry yellow-green
        "#17becf",
    ];
    let limits = if args.strict {
        Some((target.start, target.end))
    } else {
        None
    };
    let non_empty_blocks: Vec<_> = blocks_per_bam
        .iter()
        .filter(|blocks| !blocks[0].empty)
        .collect();
    for (height, blocks) in non_empty_blocks.iter().enumerate() {
        let mut show_legend = true;
        for (block, color) in blocks.iter().zip(default_colors.iter().cycle()) {
            plot.add_trace(block.plot(height, color.to_string(), show_legend, args.width, limits));
            show_legend = false;
        }
    }
    if let Some(p) = args.bed.clone() {
        log::debug!("Processing bed annotation file: {:?}", p);
        for annot_interval in annot::parse_bed(p, &target)
            .expect("Failure when parsing annotation from bed file")
            .into_iter()
        {
            log::debug!("Adding annotation trace");
            plot.add_trace(annot_interval.plot())
        }
        log::debug!("Bed annotations added");
    }
    log::debug!("Setting plot layout");
    plot.set_layout(
        Layout::new()
            .title(format!("Phase block map {}", args.region))
            .y_axis(
                Axis::new()
                    .show_line(false)
                    .title("Individuals".to_string())
                    .show_grid(false)
                    .show_tick_labels(false)
                    .show_spikes(false),
            )
            .height(1000)
            .legend(Legend::new().trace_group_gap(0)),
    );
    log::debug!("Writing HTML output to: {}", args.output);
    plot.write_html(args.output.clone());
    log::debug!("HTML output written successfully");
}

#[cfg(test)]
#[ctor::ctor]
fn init() {
    env_logger::init();
}

#[test]
fn verify_app() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}

#[test]
fn run() {
    let test_cli = Cli {
        input: vec![
            PathBuf::from("test-data/small-test-phased.bam"),
            PathBuf::from("test-data/small-test-phased.bam"),
            PathBuf::from("test-data/small-test-phased.bam"),
        ],
        bed: None,
        threads: 2,
        decompression: 1,
        output: "test.html".to_string(),
        region: "chr7:152743763-156779243".to_string(),
        width: None,
        summary: None,
        strict: false,
    };
    run_phasius(test_cli);
}

#[test]
fn run_with_width() {
    let test_cli = Cli {
        input: vec![
            PathBuf::from("test-data/small-test-phased.bam"),
            PathBuf::from("test-data/small-test-phased.bam"),
            PathBuf::from("test-data/small-test-phased.bam"),
        ],
        bed: None,
        threads: 2,
        decompression: 1,
        output: "test.html".to_string(),
        region: "chr7:152743763-156779243".to_string(),
        width: Some(4),
        summary: None,
        strict: false,
    };
    run_phasius(test_cli);
}

#[test]
fn run_with_commas() {
    let test_cli = Cli {
        input: vec![
            PathBuf::from("test-data/small-test-phased.bam"),
            PathBuf::from("test-data/small-test-phased.bam"),
            PathBuf::from("test-data/small-test-phased.bam"),
            PathBuf::from("test-data/small-test-phased.bam"),
        ],
        bed: None,
        threads: 2,
        decompression: 1,
        output: "test.html".to_string(),
        region: "chr7:152,743,763-156,779,243".to_string(),
        width: None,
        summary: None,
        strict: false,
    };
    run_phasius(test_cli);
}

#[test]
fn run_with_summary() {
    let test_cli = Cli {
        input: vec![
            PathBuf::from("test-data/small-test-phased.bam"),
            PathBuf::from("test-data/small-test-phased.bam"),
            PathBuf::from("test-data/small-test-phased.bam"),
        ],
        bed: None,
        threads: 2,
        decompression: 1,
        output: "test.html".to_string(),
        region: "chr7:152743763-156779243".to_string(),
        width: None,
        summary: Some("test_summary.txt".to_string()),
        strict: false,
    };
    run_phasius(test_cli);
}

#[test]
fn run_with_strict() {
    let test_cli = Cli {
        input: vec![
            PathBuf::from("test-data/small-test-phased.bam"),
            PathBuf::from("test-data/small-test-phased.bam"),
            PathBuf::from("test-data/small-test-phased.bam"),
        ],
        bed: None,
        threads: 2,
        decompression: 1,
        output: "test_strict.html".to_string(),
        region: "chr7:152800000-156700000".to_string(),
        width: None,
        summary: None,
        strict: true,
    };
    run_phasius(test_cli);
}

#[test]
fn run_without_strict() {
    let test_cli = Cli {
        input: vec![
            PathBuf::from("test-data/small-test-phased.bam"),
            PathBuf::from("test-data/small-test-phased.bam"),
            PathBuf::from("test-data/small-test-phased.bam"),
        ],
        bed: None,
        threads: 2,
        decompression: 1,
        output: "test_without_strict.html".to_string(),
        region: "chr7:152800000-156700000".to_string(),
        width: None,
        summary: None,
        strict: false,
    };
    run_phasius(test_cli);
}

#[test]
fn run_with_empty_blocks() {
    use crate::blocks::Blocks;

    let test_cli = Cli {
        input: vec![
            PathBuf::from("test-data/small-test-phased.bam"),
            PathBuf::from("test-data/small-test-phased.bam"),
        ],
        bed: None,
        threads: 2,
        decompression: 1,
        output: "test_with_empty_blocks.html".to_string(),
        region: "chr7:152800000-156700000".to_string(),
        width: None,
        summary: Some("test_empty_blocks_summary.txt".to_string()),
        strict: false,
    };

    // Extract blocks from BAM files
    let target = utils::process_region(&test_cli.region).expect("Error: Improper interval!");
    let mut blocks_per_bam = extract_blocks(&test_cli, &target);

    // Add a single empty block
    blocks_per_bam.push(vec![Blocks {
        start: 0,
        end: 0,
        name: "test-data/empty-test.bam".to_string(),
        empty: true,
    }]);

    // Test plotting
    plot_blocks(&blocks_per_bam, &test_cli, target);

    // Test summarizing
    let summary_per_sample = summary::summarize(&blocks_per_bam);
    std::fs::write(test_cli.summary.unwrap(), summary_per_sample).expect("Unable to write file");

    // Verify the summary file was created
    assert!(std::path::Path::new("test_empty_blocks_summary.txt").exists());
}
