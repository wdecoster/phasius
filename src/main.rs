use clap::AppSettings::DeriveDisplayOrder;
use clap::Parser;
use log::{error, info};
use plotly::common::Mode;
use plotly::common::Title;
use plotly::layout::Axis;
use plotly::layout::Legend;
use plotly::{Layout, Plot, Scatter};

use std::path::PathBuf; // for BAM_F*

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

    /// Number of parallel decompression threads to use
    #[clap(short, long, value_parser, default_value_t = 4)]
    threads: usize,

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
    let target = match utils::process_region(&args.region) {
        Ok(reg) => reg,
        Err(e) => {
            error!("Improper interval {}\n{}", args.region, e);
            panic!();
        }
    };
    info!("Collected arguments");
    let input = args.input.clone();
    let blocks_per_bam = input.iter().map(|b| {
        extract_from_bam::blocks_from_bam(b, args.threads, &target)
            .expect("Failure when parsing region from file.")
    });

    let mut plot = Plot::new();
    for (index, (blocks, name)) in blocks_per_bam.zip(args.input).enumerate() {
        let trace_name = name.file_stem().unwrap().to_str().unwrap();
        let mut show_legend = true;
        for (begin, end) in blocks {
            let trace1 = Scatter::new(vec![begin, end], vec![index, index])
                .mode(Mode::Lines)
                .name(trace_name)
                .legend_group(trace_name)
                .show_legend(show_legend);

            plot.add_trace(trace1);
            show_legend = false;
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
