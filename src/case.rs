use std::{collections::HashMap, cell::RefCell, rc::Rc, fs};

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
    fn new(mass: f64, position: [f64; 2], velocity: Option<[f64; 2]>, parent_name: Option<String>) -> Self {
        Self { mass, position, velocity, parent_name }
    }

    fn to_object_stationary(&self, name: String) -> Object {
        let position = DVec2::new(self.position[0], self.position[1]);
        Object::new_stationary(name, self.mass, position)
    }

    fn to_object_orbit(&self, name: String, parent: Rc<RefCell<Object>>) -> Object {
        let position = DVec2::new(self.position[0], self.position[1]);
        let velocity = self.velocity.expect(format!("Object {} has a parent but no mass", name).as_str());
        let velocity = DVec2::new(velocity[0], velocity[1]);
        Object::new_orbit(name, self.mass, position, velocity, parent)
    }
}

#[derive(Deserialize, Serialize)]
pub struct CaseData {
    end_time: f64,
    time_step: f64,
    objects: HashMap<String, ObjectData>,
    encounters: Vec<Encounter>,
}

impl CaseData {
    pub fn get_objects(&self) -> Vec<Rc<RefCell<Object>>> {
        let mut real_objects: HashMap<String, Rc<RefCell<Object>>> = HashMap::new();
        while real_objects.len() != self.objects.len() {
            for (name, object) in &self.objects {
                if real_objects.contains_key(name) {
                    continue;
                }
                if let Some(parent_name) = &object.parent_name {
                    if let Some(parent) = real_objects.get(parent_name) {
                        let real_object = Rc::new(RefCell::new(object.to_object_orbit(name.clone(), parent.clone())));
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
}

pub fn load_case_data(name: String) -> CaseData {
    let file = fs::read_to_string("cases/".to_string() + name.as_str() + ".json").expect(format!("Failed to load test case {}", name).as_str());
    serde_json::from_str(file.as_str()).expect(format!("Failed to deserialize test case {}", name).as_str())
}

