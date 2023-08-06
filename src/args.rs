use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "A tool to parse vtfx files (from x360 and ps3)")]
pub struct Args {
    /// Input .vtf file path
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output folder
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Try to output only mip 0
    #[arg(long, default_value_t = false)]
    pub mip0_only: bool,

    /// Do not use big to little endian fix on DXT images
    #[arg(long, default_value_t = false)]
    pub no_dxt_fix: bool,

    /// Do not export any resources
    #[arg(long, default_value_t = false)]
    pub no_resource_export: bool,

    /// Auto open exported images
    #[arg(long, default_value_t = false)]
    pub open: bool,
}