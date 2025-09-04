use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// A comma-separated list of peripheral names to translate.  If this vector is empty, then all
    /// peripherals will be translated.
    #[clap(long)]
    only: Vec<String>,

    /// The path of the SVD file to parse
    svd_file: String,

    /// The path of the Rust file to output
    rust_file: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut builder = svdgen::Builder::default().svd_file(&args.svd_file);

    for name in args.only {
        builder = builder.include(&name);
    }

    let device = builder.build()?;
    device.write_to_file(&args.rust_file)?;

    Ok(())
}
