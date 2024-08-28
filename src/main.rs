mod app;
mod colors;
mod raytracer;
mod texture;

use anyhow::Result;
use app::Application;
use clap::Parser;
use log::trace;
use raytracer::RenderMode;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, default_value = "gpu")]
    mode: RenderMode,
    #[arg(short, long, default_value = "128")]
    samples: u32,
}

fn main() -> Result<()> {
    env_logger::builder().format_timestamp(None).init();
    trace!("Initialized logger");

    let args = Args::parse();
    trace!("Parsed args");

    pollster::block_on(Application::run(args))?;

    Ok(())
}
