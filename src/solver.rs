use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{case::{load_case_objects, CaseMetadata}, constants::MAX_MASS, object::Object};

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

fn get_entrance(objects: &Vec<Rc<RefCell<Object>>>, object: &Rc<RefCell<Object>>) -> Option<Rc<RefCell<Object>>> {
    for other_object in objects {
        if other_object.borrow().get_mass() > MAX_MASS
                && other_object.borrow().get_final_parent().is_some()
                && other_object.as_ptr() != object.as_ptr()
                && object.borrow().get_final_parent().unwrap().as_ptr() == other_object.borrow().get_final_parent().unwrap().as_ptr() {
            let soi = other_object.borrow().get_soi().unwrap();
            let distance = (object.borrow().get_final_position() - other_object.borrow().get_final_position()).magnitude();
            if distance < soi {
                return Some(other_object.clone());
            }
        }
        
    }
    None
}

fn get_exit(object: &Rc<RefCell<Object>>) -> Option<Rc<RefCell<Object>>> {
    let parent = object.borrow().get_final_parent()?;
    let soi = parent.borrow().get_soi()?;
    let distance = object.borrow().get_final_position().magnitude();
    if distance > soi {
        parent.borrow().get_final_parent()
    } else {
        None
    }
}

fn get_object_from_name(objects: &Vec<Rc<RefCell<Object>>>, name: &String) -> Rc<RefCell<Object>> {
    objects.iter()
        .find(|object| object.borrow().get_name() == *name)
        .cloned()
        .expect("Failed to find object")
}

pub fn solve(name: &String, metadata: &CaseMetadata) -> Vec<Encounter> {
    // construct all the objects again rather than using existing ones
    // this prevents us from messing around with the main simulation while solving
    let objects = load_case_objects(name, metadata); 
    let mut time = 0.0;
    let mut encounters = vec![];
    while time < metadata.get_end_time() {
        let mut new_encounters = vec![];
        for object in &objects {
            if object.borrow().get_final_parent().is_none() {
                continue;
            }
            if let Some(new_parent) = get_entrance(&objects, object) {
                new_encounters.push(Encounter {
                    encounter_type: EncounterType::Entrance,
                    object: object.borrow().get_name(),
                    new_parent: new_parent.borrow().get_name(),
                    time,
                });
            }
            if let Some(new_parent) = get_exit(object) {
                new_encounters.push(Encounter {
                    encounter_type: EncounterType::Exit,
                    object: object.borrow().get_name(),
                    new_parent: new_parent.borrow().get_name(),
                    time,
                });
            }
        }
        
        for encounter in &new_encounters {
            let object = get_object_from_name(&objects, &encounter.object);
            let new_parent = get_object_from_name(&objects, &encounter.new_parent);
            match encounter.encounter_type {
                EncounterType::Entrance => {
                    let new_position = object.borrow().get_final_position() - new_parent.borrow().get_final_position();
                    let new_velocity = object.borrow().get_final_velocity() - new_parent.borrow().get_final_velocity();
                    object.borrow_mut().change_parent(new_parent, new_position, new_velocity, time);
                },
                EncounterType::Exit => {
                    let new_position = object.borrow().get_final_position() + object.borrow().get_final_parent().unwrap().borrow().get_final_position();
                    let new_velocity = object.borrow().get_final_velocity() + object.borrow().get_final_parent().unwrap().borrow().get_final_velocity();
                    object.borrow_mut().change_parent(new_parent, new_position, new_velocity, time);
                },
            }
        }
        
        encounters.append(&mut new_encounters);
        time += metadata.get_time_step();
        for object in &objects {
            if let Some(orbits) = object.borrow_mut().get_orbits_mut() {
                orbits.front_mut().unwrap().end_at(time + metadata.get_time_step());
            }
            object.borrow_mut().update_front(metadata.get_time_step());
        }
    }
    encounters
}