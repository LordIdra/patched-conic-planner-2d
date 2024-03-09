use std::{rc::Rc, cell::RefCell, time::Instant};

use clap::Parser;
use eframe::{CreationContext, egui::{CentralPanel, Context, Key, Painter, SidePanel}, Frame, epaint::{Shape, Pos2, Color32, Rect, pos2}, emath::RectTransform};
use nalgebra_glm::DVec2;

use crate::{case::{load_case_encounters, load_case_metadata, load_case_objects, save_case_encounters, CaseMetadata}, constants::{LINES_PER_ORBIT, LINE_WIDTH, OBJECT_RADIUS}, object::Object, solver::{encounter::{Encounter, EncounterType}, solve}, util::format_time};

fn dvec2_to_pos2(x: DVec2) -> Pos2 {
    // the simulation takes y as up, but the painter takes y as dwown (lol)
    pos2(x.x as f32, -x.y as f32)
}

fn line(to_screen: &RectTransform, points: [Pos2; 2], color: Color32, width: f32) -> Shape {
    Shape::line_segment([to_screen * points[0], to_screen * points[1]], (width, color))
}

fn circle(to_screen: &RectTransform, center: Pos2, color: Color32, radius: f32) -> Shape {
    Shape::circle_filled(to_screen * center, radius, color)
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

fn get_new_position_velocity_entrance(object: Rc<RefCell<Object>>, new_parent: Rc<RefCell<Object>>, encounter: &Encounter) -> (DVec2, DVec2) {
    let object_ref = object.borrow();
    let object_orbit = object_ref.get_orbits().unwrap().front().unwrap();
    let new_parent_ref = new_parent.borrow();
    let new_parent_orbit = new_parent_ref.get_orbits().unwrap().front().unwrap();

    let object_position = object_orbit.get_position_from_theta(object_orbit.get_theta_from_time(encounter.get_time()));
    let new_parent_position = new_parent_orbit.get_position_from_theta(new_parent_orbit.get_theta_from_time(encounter.get_time()));
    let object_velocity = object_orbit.get_velocity_from_theta(object_orbit.get_theta_from_time(encounter.get_time()));
    let new_parent_velocity = new_parent_orbit.get_velocity_from_theta(new_parent_orbit.get_theta_from_time(encounter.get_time()));

    (object_position - new_parent_position, object_velocity - new_parent_velocity)
}

fn get_new_position_velocity_exit(object: Rc<RefCell<Object>>, encounter: &Encounter) -> (DVec2, DVec2) {
    let object_ref = object.borrow();
    let object_orbit = object_ref.get_orbits().unwrap().front().unwrap();
    let old_parent = object_orbit.get_parent();
    let old_parent_ref = old_parent.borrow();
    let old_parent_orbit = old_parent_ref.get_orbits().unwrap().front().unwrap();

    let object_position = object_orbit.get_position_from_theta(object_orbit.get_theta_from_time(encounter.get_time()));
    let old_parent_position = old_parent_orbit.get_position_from_theta(old_parent_orbit.get_theta_from_time(encounter.get_time()));
    let object_velocity = object_orbit.get_velocity_from_theta(object_orbit.get_theta_from_time(encounter.get_time()));
    let old_parent_velocity = old_parent_orbit.get_velocity_from_theta(old_parent_orbit.get_theta_from_time(encounter.get_time()));

    (object_position + old_parent_position, object_velocity + old_parent_velocity)
}

fn do_encounters(objects: &Vec<Rc<RefCell<Object>>>, encounters: &Vec<Encounter>, metadata: &CaseMetadata) {
    for encounter in encounters {
        let object = objects.iter()
            .find(|object| object.borrow().get_name() == encounter.get_object())
            .cloned()
            .expect("Encounter for nonexistent object; try recomputing encounters?");
        match encounter.get_encounter_type() {
            EncounterType::Entrance => {
                let new_parent = objects.iter()
                    .find(|object| object.borrow().get_name() == encounter.get_new_parent())
                    .cloned()
                    .expect("Entrance encounter with nonexistent new parent; try recomputing encounters?");

                let (new_position, new_velocity) = get_new_position_velocity_entrance(object.clone(), new_parent.clone(), encounter);
                object.borrow_mut().change_parent(new_parent.clone(), new_position, new_velocity, encounter.get_time());
            }
            EncounterType::Exit => {
                let new_parent = objects.iter()
                    .find(|object| object.borrow().get_name() == encounter.get_new_parent())
                    .cloned()
                    .expect("Exit encounter with nonexistent new parent; try recomputing encounters?");

                let (new_position, new_velocity) = get_new_position_velocity_exit(object.clone(), encounter);
                object.borrow_mut().change_parent(new_parent.clone(), new_position, new_velocity, encounter.get_time());
            }
        }
    }

    for object in objects {
        if let Some(orbits) = object.borrow_mut().get_orbits_mut() {
            orbits.front_mut().unwrap().end_at(metadata.get_end_time());
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: String,
}

pub struct State {
    name: String,
    time: f64,
    last_frame_time: Instant,
    zoom: f32,
    speed: f64,
    focus: Rc<RefCell<Object>>,
    metadata: CaseMetadata,
    objects: Vec<Rc<RefCell<Object>>>,
}

impl State {
    pub fn new(_: &CreationContext) -> Self {
        let time = 0.0;
        let last_frame_time = Instant::now();
        let speed = 1.0;
        let args = Args::parse();
        let name = args.name;
        let metadata = load_case_metadata(&name);
        let objects = load_case_objects(&name, &metadata);
        if let Ok(encounters) = load_case_encounters(&name) {
            do_encounters(&objects, &encounters, &metadata);
        }
        let zoom = metadata.get_starting_zoom();
        let focus = objects.iter()
            .find(|object| object.borrow().get_name() == metadata.get_focus())
            .expect("Object to focus does not exist")
            .clone();
        Self { name, time, last_frame_time, zoom, speed, focus, metadata, objects }
    }

    fn draw_orbits(&self, screen_rect: Rect) -> Vec<Shape> {
        let translation = dvec2_to_pos2(self.focus.borrow().get_current_absolute_position());
        let mut lines: Vec<Shape> = Vec::new();
        let to_screen = RectTransform::from_to(Rect::from_center_size(translation, screen_rect.square_proportions() / self.zoom), screen_rect);
        let colors = orbit_colors();
        for object in &self.objects {
            let mut color_index = 0;
            if let Some(orbits) = object.borrow().get_orbits() {
                for orbit in orbits {
                    let parent_position = orbit.get_parent().borrow().get_current_absolute_position();
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

    fn draw_objects(&self, screen_rect: Rect) -> Vec<Shape> {
        let translation = dvec2_to_pos2(self.focus.borrow().get_current_absolute_position());
        let mut circles: Vec<Shape> = Vec::new();
        let to_screen = RectTransform::from_to(Rect::from_center_size(translation, screen_rect.square_proportions() / self.zoom), screen_rect);
        for object in &self.objects {
            circles.push(circle(&to_screen, dvec2_to_pos2(object.borrow().get_current_absolute_position()), Color32::from_rgb(255, 255, 255), OBJECT_RADIUS));
        }
        circles
    }

    fn reload(&mut self) {
        self.time = 0.0;
        self.speed = 1.0;
        self.metadata = load_case_metadata(&self.name);
        self.objects = load_case_objects(&self.name, &self.metadata);
        self.zoom = self.metadata.get_starting_zoom();
        self.focus = self.objects.iter()
            .find(|object| object.borrow().get_name() == self.metadata.get_focus())
            .expect("Object to focus does not exist")
            .clone();
    }
}

impl eframe::App for State {
    fn update(&mut self, context: &Context, _frame: &mut Frame) {
        let delta_time = (Instant::now() - self.last_frame_time).as_secs_f64();
        let simulated_delta_time = delta_time * self.speed;
        self.time += simulated_delta_time;
        self.last_frame_time = Instant::now();

        for object in &self.objects {
            object.borrow_mut().update_back(simulated_delta_time);
        }

        SidePanel::left("main").show(context, |ui| {
            ui.label(format!("{} FPS", f64::round(1.0 / delta_time)));
            ui.label(format!("Time: {}", format_time(self.time)));
            ui.label(format!("End: {}", format_time(self.metadata.get_end_time())));
            ui.label(format!("Solver step: {}", format_time(self.metadata.get_time_step())));

            if ui.button("Refresh").clicked() {
                self.reload();
                let encounters = solve(&self.name, &self.metadata);
                save_case_encounters(&self.name, &encounters);
                if let Ok(encounters) = load_case_encounters(&self.name) {
                    do_encounters(&self.objects, &encounters, &self.metadata);
                }
            }
        });

        CentralPanel::default().show(context, |ui| {
            let painter = Painter::new(
                ui.ctx().clone(),
                ui.layer_id(),
                ui.available_rect_before_wrap(),
            );
            painter.extend(self.draw_orbits(context.screen_rect()));
            painter.extend(self.draw_objects(context.screen_rect()));
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