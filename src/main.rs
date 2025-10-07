#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use clap::Parser;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::{app::App, emulator::Quirks};

mod app;
mod emulator;
mod memory;
mod stack;
mod virtual_buffer;

#[cfg(feature = "audio")]
mod sound;

fn parse_quirk(s: &str) -> Result<Quirks, String> {
    match s.to_lowercase().as_str() {
        "vf-reset" => Ok(Quirks::VF_RESET),
        "memory" => Ok(Quirks::MEMORY),
        "display-wait" => Ok(Quirks::DISPLAY_WAIT),
        "clipping" => Ok(Quirks::CLIPPING),
        "shifting" => Ok(Quirks::SHIFTING),
        "jumping" => Ok(Quirks::JUMPING),
        _ => Err(format!("`{}` is not a valid quirk identifier", s)),
    }
}

/// Defines this program's command-line arguments
#[derive(Parser, Debug)]
struct Args {
    /// Path to the CHIP-8 ROM to load
    #[arg(index = 1)]
    input_file: String,

    /// Set custom CHIP-8 quirks. Can be repeated.
    /// Options: vf-reset, memory, clipping, shifting, jumping, display-wait
    #[arg(long="quirk", short='q', value_name="QUIRK_NAME", value_parser = parse_quirk)]
    quirks: Vec<Quirks>,
}

fn main() {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "chip8_rs=info");
    env_logger::init_from_env(env);

    let args = Args::parse();

    let custom_quirks = if args.quirks.is_empty() {
        None
    } else {
        Some(
            args.quirks
                .into_iter()
                .fold(Quirks::empty(), |acc, quirk| acc | quirk),
        )
    };

    if let Some(q) = custom_quirks {
        log::info!("Using custom quirks: {:?}", q);
    }

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

    let mut app = App::new(data, custom_quirks);
    if let Err(e) = event_loop.run_app(&mut app) {
        log::error!("Error running event loop: {:?}", e);
        std::process::exit(1);
    }
}
