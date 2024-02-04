use std::{cell::RefCell, collections::HashMap, fs, io, rc::Rc};

use nalgebra_glm::DVec2;
use serde::{Deserialize, Serialize};

use crate::{object::Object, solver::Encounter};

#[derive(Deserialize, Serialize)]
struct ObjectData {
    mass: f64,
    position: [f64; 2],
    velocity: Option<[f64; 2]>,
    parent_name: Option<String>,
}

impl ObjectData {
    fn to_object_stationary(&self, name: String) -> Object {
        let position = DVec2::new(self.position[0], self.position[1]);
        Object::new_stationary(name, self.mass, position)
    }

    fn to_object_orbit(&self, name: String, parent: Rc<RefCell<Object>>, end_time: f64) -> Object {
        let position = DVec2::new(self.position[0], self.position[1]);
        let velocity = self.velocity.expect(format!("Object {} has a parent but no mass", name).as_str());
        let velocity = DVec2::new(velocity[0], velocity[1]);
        Object::new_orbit(name, self.mass, position, velocity, parent, end_time)
    }
}

#[derive(Deserialize, Serialize)]
pub struct CaseMetadata {
    end_time: f64,
    time_step: f64,
    focus: String,
    starting_zoom: f32,
}

impl CaseMetadata {
    pub fn get_end_time(&self) -> f64 {
        self.end_time
    }

    pub fn get_time_step(&self) -> f64 {
        self.time_step
    }

    pub fn get_focus(&self) -> String {
        self.focus.clone()
    }

    pub fn get_starting_zoom(&self) -> f32 {
        self.starting_zoom
    }
}

fn get_objects_from_object_data(objects: HashMap<String, ObjectData>, end_time: f64) -> Vec<Rc<RefCell<Object>>> {
    let mut real_objects: HashMap<String, Rc<RefCell<Object>>> = HashMap::new();
    while real_objects.len() != objects.len() {
        for (name, object) in &objects {
            if real_objects.contains_key(name) {
                continue;
            }
            if let Some(parent_name) = &object.parent_name {
                if let Some(parent) = real_objects.get(parent_name) {
                    let real_object = Rc::new(RefCell::new(object.to_object_orbit(name.clone(), parent.clone(), end_time)));
                    real_objects.insert(name.clone(), real_object);
                }
            } else {
                let real_object = Rc::new(RefCell::new(object.to_object_stationary(name.clone())));
                real_objects.insert(name.clone(), real_object);
            }
        }
    }
    real_objects.values().cloned().collect()
}

pub fn load_case_metadata(name: &String) -> CaseMetadata {
    let file = fs::read_to_string("cases/".to_string() + name.as_str() + "/metadata.json")
        .expect(format!("Failed to load metadata {}", name).as_str());
    serde_json::from_str(file.as_str())
        .expect(format!("Failed to deserialize metadata {}", name).as_str())
}

pub fn load_case_objects(name: &String, metadata: &CaseMetadata) -> Vec<Rc<RefCell<Object>>> {
    let file = fs::read_to_string("cases/".to_string() + name.as_str() + "/objects.json")
        .expect(format!("Failed to load objects {}", name).as_str());
    let object_data = serde_json::from_str(file.as_str())
        .expect(format!("Failed to deserialize objects {}", name).as_str());
    get_objects_from_object_data(object_data, metadata.end_time)
}

pub fn load_case_encounters(name: &String) -> Result<Vec<Encounter>, io::Error> {
    let file = fs::read_to_string("cases/".to_string() + name.as_str() + "/encounters.json")?;
    Ok(serde_json::from_str(file.as_str())
        .expect(format!("Failed to deserialize objects {}", name).as_str()))
}

pub fn save_case_encounters(name: &String, encounters: &Vec<Encounter>) {
    let json = serde_json::to_string(encounters)
        .expect(format!("Failed to serialize encounters {}", name).as_str());
    fs::write("cases/".to_string() + name.as_str() + "/encounters.json", json)
        .expect(format!("Failed to save encounters {}", name).as_str());
}
