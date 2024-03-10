use std::{fs, process::exit};

use clap::Parser;
use eframe::{NativeOptions, Renderer, run_native, CreationContext};
use headless::SimulationState;
use state::State;

mod bisection;
mod case;
mod constants;
mod headless;
mod object;
mod solver;
mod state;
mod util;

fn recompute_all() {
    for entry in fs::read_dir("cases").unwrap() {
        let entry = entry.unwrap();
        let path = entry.file_name();
        let name = path.to_str().unwrap().to_string();
        println!("Recomputing {}", name.clone());
        SimulationState::new(name, 0.0).reload();
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    recompute_all: bool,
    #[arg(short, long)]
    name: Option<String>,
}

fn create_app(creation_context: &CreationContext) -> Box<dyn eframe::App> {
    let args = Args::parse();
    if args.recompute_all {
        recompute_all();
        exit(0);
    }
    if args.name.is_none() {
        println!("A --name must be provided if not using --recompute-all");
        exit(0);
    }
    Box::new(State::new(creation_context, args.name.unwrap()))
}

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        renderer: Renderer::Glow,
        multisampling: 16,
        ..Default::default()
    };
    
    run_native("Conic Planner", options, Box::new(create_app))
}
