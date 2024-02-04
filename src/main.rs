use eframe::{NativeOptions, Renderer, run_native, CreationContext};
use state::State;

mod case;
mod constants;
mod object;
mod solver;
mod state;
mod util;

fn create_app(creation_context: &CreationContext) -> Box<dyn eframe::App> {
    Box::new(State::new(creation_context))
}

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        renderer: Renderer::Glow,
        multisampling: 16,
        ..Default::default()
    };
    
    run_native("Conic Planner", options, Box::new(create_app))
}
