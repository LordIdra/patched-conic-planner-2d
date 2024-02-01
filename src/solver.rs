use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{case::CaseData, constants::MAX_MASS, object::Object};

#[derive(Deserialize, Serialize)]
pub enum EncounterType {
    Entrance,
    Exit,
}

#[derive(Deserialize, Serialize)]
pub struct Encounter {
    encounter_type: EncounterType,
    object: String,
    new_parent: String,
    time: f64,
}

fn get_entrance_objects(objects: &Vec<Rc<RefCell<Object>>>, object: &Rc<RefCell<Object>>) -> Vec<Rc<RefCell<Object>>> {
    let mut entrance_objects = vec![];
    for other_object in objects {
        if object.as_ptr() == other_object.as_ptr() || object.borrow().get_mass() < MAX_MASS {
            continue;
        }
        if other_object.borrow().get_soi().is_some() {
            entrance_objects.push(other_object.clone());
        }
    }
    entrance_objects
}

fn get_entrance(objects: &Vec<Rc<RefCell<Object>>>, object: &Rc<RefCell<Object>>) -> Option<Rc<RefCell<Object>>> {
    for other_object in &get_entrance_objects(objects, object) {
        let soi = object.borrow().get_soi().unwrap();
        let distance = object.borrow().get_position().magnitude();
        if distance < soi {
            return Some(other_object.clone());
        }
    }
    None
}

fn get_exit(object: &Rc<RefCell<Object>>) -> Option<Rc<RefCell<Object>>> {
    let parent = object.borrow().get_parent()?;
    let soi = parent.borrow().get_soi()?;
    let distance = object.borrow().get_position().magnitude();
    if distance > soi {
        Some(parent.clone())
    } else {
        None
    }
}

pub fn solve(case_data: CaseData) -> Vec<Encounter> {
    let objects = case_data.get_objects();
    let time = 0.0;
    let mut encounters = vec![];
    while time < SIMULATION_END_TIME {
        for object in &objects {
            if object.borrow().get_mass() < MAX_MASS {
                if let Some(new_parent) = get_entrance(&objects, object) {
                    encounters.push(Encounter {
                        encounter_type: EncounterType::Entrance,
                        object: object.borrow().get_name(),
                        new_parent: new_parent.borrow().get_name(),
                        time,
                    })
                }
                if let Some(new_parent) = get_exit(object) {
                    encounters.push(Encounter {
                        encounter_type: EncounterType::Exit,
                        object: object.borrow().get_name(),
                        new_parent: new_parent.borrow().get_name(),
                        time,
                    })
                }
            }
        }
        time += SIM
    }
    encounters
}