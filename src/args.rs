use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "A tool to parse vtfx files (from x360 and ps3)")]
pub struct Args {
    /// Input path (process single file) or folder (processes all vtf files in folder)
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output folder for exported images
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// If alpha should be included in image export
    #[arg(long, default_value_t = false)]
    pub export_alpha: bool,

    /// Force apply big to little endian fix on DXT image resources (otherwise automatic)
    #[arg(long, default_value_t = false)]
    pub force_dxt_endian_fix: bool,

    /// Do not export any resources
    #[arg(long, default_value_t = false)]
    pub no_resource_export: bool,

    /// Auto open exported images
    #[arg(long, default_value_t = false)]
    pub open: bool
}