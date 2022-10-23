use clap::AppSettings::DeriveDisplayOrder;
use clap::Parser;
use log::info;
use plotly::common::Title;
use plotly::layout::{Axis, Legend};
use plotly::{Layout, Plot};
use rayon::prelude::*;
use std::path::PathBuf;

pub mod annot;
pub mod extract_from_bam;
pub mod utils;

// The arguments end up in the Cli struct
#[derive(Parser, Debug)]
#[structopt(global_settings=&[DeriveDisplayOrder])]
#[clap(author, version, about="Tool to draw a map of phaseblocks across crams/bams", long_about = None)]
struct Cli {
    /// cram or bam files to check
    // this validator gets applied to each element from the Vec separately
    #[clap(parse(from_os_str), multiple_values = true, required = true, validator=is_file)]
    input: Vec<PathBuf>,

    /// bed file annotation to use (bgzipped and tabix indexed)
    #[clap(short, long, parse(from_os_str), validator=is_file)]
    bed: Option<PathBuf>,

    /// Number of crams/bams to parse in parallel
    #[clap(short, long, value_parser, default_value_t = 4)]
    threads: usize,

    /// Number of decompression threads to use per cram/bam
    #[clap(short, long, value_parser, default_value_t = 1)]
    decompression: usize,

    /// HTML output file name
    #[clap(short, long, value_parser)]
    output: String,

    /// region string to plot phase blocks from
    #[clap(short, long, value_parser)]
    region: String,
}

fn is_file(pathname: &str) -> Result<(), String> {
    let path = PathBuf::from(pathname);
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("Input file {} is invalid", path.display()))
    }
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    let target = utils::process_region(&args.region).expect("Error: Improper interval!");

    info!("Collected arguments");
    let input = args.input.clone();
    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build()
        .unwrap();
    let blocks_per_bam: Vec<Vec<extract_from_bam::Blocks>> = input
        .into_par_iter()
        .map(|b| {
            extract_from_bam::blocks_from_bam(&b, args.decompression, &target)
                .expect("Failure when parsing region from bam file.")
        })
        .collect();

    let mut plot = Plot::new();
    let default_colors = vec![
        "#1f77b4", // muted blue
        "#ff7f0e", // safety orange
        "#2ca02c", // cooked asparagus green
        "#d62728", // brick red
        "#9467bd", // muted purple
        "#8c564b", // chestnut brown
        "#e377c2", // raspberry yogurt pink
        "#7f7f7f", // middle gray
        "#bcbd22", // curry yellow-green
        "#17becf", // blue-teal
    ];
    for (height, blocks) in blocks_per_bam.iter().enumerate() {
        let mut show_legend = true;
        for (block, color) in blocks.iter().zip(default_colors.iter().cycle()) {
            plot.add_trace(block.plot(height, color.to_string(), show_legend));
            show_legend = false;
        }
    }
    if let Some(p) = args.bed {
        for annot_interval in annot::parse_bed(p, &target)
            .expect("Failure when parsing annotation from bed file")
            .into_iter()
        {
            plot.add_trace(annot_interval.plot())
        }
    }
    plot.set_layout(
        Layout::new()
            .title(Title::new(&format!("Phase block map {}", args.region)))
            .y_axis(
                Axis::new()
                    .show_line(false)
                    .title("Individuals".into())
                    .show_grid(false)
                    .show_tick_labels(false)
                    .show_spikes(false),
            )
            .height(1000)
            .legend(Legend::new().trace_group_gap(0)),
    );
    plot.write_html(args.output);
}
