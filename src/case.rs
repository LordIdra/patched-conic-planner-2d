use std::{collections::HashMap, cell::RefCell, rc::Rc, fs};

use nalgebra_glm::DVec2;
use serde::Deserialize;

use crate::object::Object;

#[derive(Deserialize)]
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

#[derive(Deserialize)]
struct CaseData {
    object_data_map: HashMap<String, ObjectData>
}

impl CaseData {
    fn get_objects(self) -> Vec<Rc<RefCell<Object>>> {
        let mut object_map: HashMap<String, Rc<RefCell<Object>>> = HashMap::new();
        while object_map.len() != self.object_data_map.len() {
            for (name, object_data) in &self.object_data_map {
                if object_map.contains_key(name) {
                    continue;
                }
                if let Some(parent_name) = &object_data.parent_name {
                    if let Some(parent) = object_map.get(parent_name) {
                        let object = Rc::new(RefCell::new(object_data.to_object_orbit(name.clone(), parent.clone())));
                        object_map.insert(name.clone(), object);
                    }
                } else {
                    let object = Rc::new(RefCell::new(object_data.to_object_stationary(name.clone())));
                    object_map.insert(name.clone(), object);
                }
            }
        }
        object_map.values().cloned().collect()
    }
}

pub fn load_objects(name: String) -> Vec<Rc<RefCell<Object>>> {
    let file = fs::read_to_string("cases/".to_string() + name.as_str() + "/entities.json").expect(format!("Failed to load test case {}", name).as_str());
    let case_data: CaseData = serde_json::from_str(file.as_str()).expect(format!("Failed to deserialize test case {}", name).as_str());
    case_data.get_objects()
}

