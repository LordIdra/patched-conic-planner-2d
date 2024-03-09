use std::{cell::RefCell, rc::Rc};

use crate::{case::{load_case_objects, CaseMetadata}, object::Object};

use self::encounter::{get_entrance, get_exit, Encounter, EncounterType};

pub mod encounter;

fn get_object_from_name(objects: &Vec<Rc<RefCell<Object>>>, name: &String) -> Rc<RefCell<Object>> {
    objects.iter()
        .find(|object| object.borrow().get_name() == *name)
        .cloned()
        .expect("Failed to find object")
}

fn do_entrance(object: Rc<RefCell<Object>>, new_parent: Rc<RefCell<Object>>, time: f64) {
    let new_position = object.borrow().get_final_position() 
        - new_parent.borrow().get_final_position();
    let new_velocity = object.borrow().get_final_velocity() 
        - new_parent.borrow().get_final_velocity();
    object.borrow_mut().change_parent(new_parent, new_position, new_velocity, time);
}

fn do_exit(object: Rc<RefCell<Object>>, new_parent: Rc<RefCell<Object>>, time: f64) {
    let new_position = object.borrow().get_final_position() 
        + object.borrow().get_final_parent().unwrap().borrow().get_final_position();
    let new_velocity = object.borrow().get_final_velocity() 
        + object.borrow().get_final_parent().unwrap().borrow().get_final_velocity();
    object.borrow_mut().change_parent(new_parent, new_position, new_velocity, time);
}

fn step_time(objects: &Vec<Rc<RefCell<Object>>>, time: f64, time_step: f64) {
    for object in objects {
        if let Some(orbits) = object.borrow_mut().get_orbits_mut() {
            orbits.front_mut().unwrap().end_at(time);
        }
        object.borrow_mut().update_front(time_step);
    }
}

pub fn solve(name: &String, metadata: &CaseMetadata) -> Vec<Encounter> {
    // construct all the objects again rather than using existing ones
    // this prevents us from messing around with the main simulation while solving
    let objects = load_case_objects(name, metadata); 
    let mut time = 0.0;
    let mut encounters = vec![];
    while time < metadata.get_end_time() {
        let mut encounter = None;
        for object in &objects {
            if object.borrow().get_final_parent().is_none() {
                continue;
            }
            if let Some(new_encounter) = get_entrance(&objects, object, time, metadata.get_time_step()) {
                encounter = Some(new_encounter);
            }
            if let Some(new_encounter) = get_exit(object, time, metadata.get_time_step()) {
                encounter = Some(new_encounter);
            }
        }

        if let Some(encounter) = encounter {
            let time_step = encounter.get_time() - time;
            time += time_step;
            step_time(&objects, time, time_step);

            let object = get_object_from_name(&objects, &encounter.get_object());
            let new_parent = get_object_from_name(&objects, &encounter.get_new_parent());
            match encounter.get_encounter_type() {
                EncounterType::Entrance => do_entrance(object, new_parent, time),
                EncounterType::Exit => do_exit(object, new_parent, time),
            }
            encounters.push(encounter);
        }
        
        time += metadata.get_time_step();
        step_time(&objects, time, metadata.get_time_step());
    }
    encounters
}