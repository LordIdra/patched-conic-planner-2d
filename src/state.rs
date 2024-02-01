use std::{rc::Rc, cell::RefCell, time::Instant};

use clap::Parser;
use eframe::{CreationContext, egui::{Context, CentralPanel, Painter, Key}, Frame, epaint::{Shape, Pos2, Color32, Rect, pos2}, emath::RectTransform};
use nalgebra_glm::DVec2;

use crate::{case::load_case_data, object::Object, util::format_time};

fn dvec2_to_pos2(x: DVec2) -> Pos2 {
    // the simulation takes y as up, but the painter takes y as dwown (lol)
    pos2(x.x as f32, -x.y as f32)
}

fn line(to_screen: &RectTransform, points: [Pos2; 2], color: Color32, width: f32) -> Shape {
    Shape::line_segment([to_screen * points[0], to_screen * points[1]], (width, color))
}

fn orbit_colors() -> Vec<Color32> {
    vec![
        Color32::from_rgb(255, 0, 0),
        Color32::from_rgb(0, 255, 0),
        Color32::from_rgb(0, 0, 255),
        Color32::from_rgb(255, 255, 0),
        Color32::from_rgb(255, 0, 255),
        Color32::from_rgb(0, 255, 255),
        Color32::from_rgb(255, 255, 255),
    ]
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: String,
    #[arg(short, long)]
    focus: String,
}

pub struct State {
    time: f64,
    last_frame_time: Instant,
    zoom: f32,
    speed: f64,
    focus: Rc<RefCell<Object>>,
    objects: Vec<Rc<RefCell<Object>>>,
}

impl State {
    pub fn new(_: &CreationContext) -> Self {
        let time = 0.0;
        let last_frame_time = Instant::now();
        let zoom = 1.0e-12;
        let speed = 1.0;
        let args = Args::parse();
        let case_data = load_case_data(args.name);
        let objects = case_data.get_objects();
        let focus = objects.iter()
            .find(|object| object.borrow().get_name() == args.focus)
            .expect("Object to focus does not exist")
            .clone();
        Self { time, last_frame_time, zoom, speed, focus, objects }
    }

    fn draw_orbits(&self, screen_rect: Rect) -> Vec<Shape> {
        let translation = dvec2_to_pos2(self.focus.borrow().get_absolute_position());
        let mut lines: Vec<Shape> = Vec::new();
        let to_screen = RectTransform::from_to(Rect::from_center_size(translation, screen_rect.square_proportions() / self.zoom), screen_rect);
        let colors = orbit_colors();
        for object in &self.objects {
            let mut color_index = 0;
            if let Some(orbits) = object.borrow().get_orbits() {
                for orbit in orbits {
                    let parent_position = orbit.get_parent().borrow().get_absolute_position();
                    let end_angle = orbit.get_end_point().get_theta();
                    let remaining_angle = orbit.get_remaining_angle();
                    let mut last_angle = end_angle;
                    for i in 0..=LINES_PER_ORBIT {
                        let angle_fraction = i as f64 / LINES_PER_ORBIT as f64;
                        let angle = end_angle - angle_fraction*remaining_angle;
                        let from = dvec2_to_pos2(parent_position + orbit.get_position_from_theta(last_angle));
                        let to = dvec2_to_pos2(parent_position + orbit.get_position_from_theta(angle));
                        lines.push(line(&to_screen, [from, to], colors[color_index], LINE_WIDTH));
                        last_angle = angle;
                    }
                    color_index += 1;
                    if color_index >= colors.len() {
                        color_index = 0;
                    }
                }
            }
        }
        lines
    }
}

impl eframe::App for State {
    fn update(&mut self, context: &Context, _frame: &mut Frame) {
        let delta_time = (Instant::now() - self.last_frame_time).as_secs_f64();
        let simulated_delta_time = delta_time * self.speed;
        self.time += simulated_delta_time;
        self.last_frame_time = Instant::now();

        for object in &self.objects {
            if let Some(orbits) = object.borrow_mut().get_orbits_mut() {
                orbits.front_mut().unwrap().update(simulated_delta_time);
                if orbits.front().unwrap().is_finished() {
                    orbits.pop_front();
                }
            }
        }

        CentralPanel::default().show(context, |ui| {
            ui.label(format!("{} FPS", f64::round(1.0 / delta_time)));
            ui.label(format!("Time: {}", format_time(self.time)));
            ui.label(format!("End: {}", format_time(SIMULATION_END_TIME)));

            let painter = Painter::new(
                ui.ctx().clone(),
                ui.layer_id(),
                ui.available_rect_before_wrap(),
            );

            painter.extend(self.draw_orbits(context.screen_rect()));
        });

        context.input(|input| {
            if input.key_pressed(Key::ArrowRight) {
                self.speed *= 5.0;
            }
            if input.key_pressed(Key::ArrowLeft) {
                self.speed /= 5.0;
            }
            self.zoom += input.scroll_delta.y * 0.005 * self.zoom;
        });

        context.request_repaint();
    }
}