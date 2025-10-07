#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use clap::Parser;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::app::App;

mod app;
mod emulator;
mod memory;
mod sound;
mod stack;
mod virtual_buffer;

/// Defines this program's command-line arguments
#[derive(Parser, Debug)]
struct Args {
    #[arg(index = 1)]
    input_file: String,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    log::info!("Loading program from: {}", args.input_file);
    let data = match std::fs::read(args.input_file) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Expected a path to a CHIP-8 program");
            log::error!("{:?}", e);
            std::process::exit(1);
        }
    };

    let event_loop = match EventLoop::new() {
        Ok(v) => v,
        Err(e) => {
            log::error!("Error creating event loop: {:?}", e);
            std::process::exit(1);
        }
    };
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(data);
    if let Err(e) = event_loop.run_app(&mut app) {
        log::error!("Error running event loop: {:?}", e);
        std::process::exit(1);
    }
}
