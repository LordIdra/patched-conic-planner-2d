use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{bisection::bisection, constants::MAX_MASS, object::Object};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum EncounterType {
    Entrance,
    Exit,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Encounter {
    encounter_type: EncounterType,
    object: String,
    new_parent: String,
    time: f64,
}

impl Encounter {
    pub fn new_entrance(object: String, new_parent: String, time: f64) -> Self {
        Self { encounter_type: EncounterType::Entrance, object, new_parent, time }
    }

    pub fn new_exit(object: String, new_parent: String, time: f64) -> Self {
        Self { encounter_type: EncounterType::Exit, object, new_parent, time }
    }

    pub fn get_encounter_type(&self) -> EncounterType {
        self.encounter_type.clone()
    }

    pub fn get_object(&self) -> String {
        self.object.clone()
    }

    pub fn get_new_parent(&self) -> String {
        self.new_parent.clone()
    }

    pub fn get_time(&self) -> f64 {
        self.time
    }
}

pub fn get_entrance(objects: &Vec<Rc<RefCell<Object>>>, object: &Rc<RefCell<Object>>, time: f64, time_step: f64) -> Option<Encounter> {
    for other_object in objects {
        if other_object.borrow().get_mass() <= MAX_MASS
                || other_object.borrow().get_final_parent().is_none()
                || other_object.as_ptr() == object.as_ptr()
                || object.borrow().get_final_parent().unwrap().as_ptr() != other_object.borrow().get_final_parent().unwrap().as_ptr() {
            continue;
        }

        let soi = other_object.borrow().get_soi().unwrap();
        let distance = (object.borrow().get_next_position(time_step) - other_object.borrow().get_next_position(time_step)).magnitude();
        if distance >= soi {
            continue;
        }

        let sdf = |t: f64| {
            let distance = (object.borrow().get_next_position(t) - other_object.borrow().get_next_position(t)).magnitude();
            distance - soi
        };

        let encounter_time = time + bisection(&sdf, 0.0, time_step);
        return Some(Encounter::new_entrance(object.borrow().get_name(), other_object.borrow().get_name(), encounter_time));
    }
    None
}

pub fn get_exit(object: &Rc<RefCell<Object>>, time: f64, time_step: f64) -> Option<Encounter> {
    let parent = object.borrow().get_final_parent()?;
    let parent_parent_name = parent.borrow().get_final_parent()?.borrow().get_name();
    let soi = parent.borrow().get_soi()?;
    let distance = object.borrow().get_final_position().magnitude();
    if distance <= soi {
        return None;
    }
    
    let sdf = |t: f64| {
        let distance = object.borrow().get_next_position(t).magnitude();
        distance - soi
    };

    let encounter_time = time + bisection(&sdf, 0.0, time_step);
    Some(Encounter::new_exit(object.borrow().get_name(), parent_parent_name, encounter_time))
}