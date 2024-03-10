use std::{cell::RefCell, rc::Rc};

use nalgebra_glm::DVec2;

use crate::{case::{load_case_encounters, load_case_metadata, load_case_objects, save_case_encounters, CaseMetadata}, object::Object, solver::{encounter::{Encounter, EncounterType}, solve}};

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

pub struct SimulationState {
    name: String,
    time: f64,
    metadata: CaseMetadata,
    objects: Vec<Rc<RefCell<Object>>>,
}

impl SimulationState {
    pub fn new(name: String, time: f64) -> Self {
        let metadata = load_case_metadata(&name);
        let objects = load_case_objects(&name, &metadata);
        let simulation_state = Self { name: name.clone(), time, metadata, objects };
        if let Ok(encounters) = load_case_encounters(&name) {
            simulation_state.do_encounters(&encounters);
        }
        simulation_state
    }

    pub fn get_time(&self) -> f64 {
        self.time
    }

    pub fn get_metadata(&self) -> &CaseMetadata {
        &self.metadata
    }

    pub fn get_objects(&self) -> &Vec<Rc<RefCell<Object>>> {
        &self.objects
    }

    pub fn update(&mut self, delta_time: f64) {
        self.time += delta_time;
        for object in &self.objects {
            object.borrow_mut().update_back(delta_time);
        }
    }

    pub fn reload(&mut self) {
        self.time = 0.0;
        self.metadata = load_case_metadata(&self.name);
        self.objects = load_case_objects(&self.name, &self.metadata);
        let encounters = solve(&self.name, &self.metadata);
        save_case_encounters(&self.name, &encounters);
        if let Ok(encounters) = load_case_encounters(&self.name) {
            self.do_encounters(&encounters);
        }
    }

    fn do_encounters(&self, encounters: &Vec<Encounter>) {
        for encounter in encounters {
            let object = self.objects.iter()
                .find(|object| object.borrow().get_name() == encounter.get_object())
                .cloned()
                .expect("Encounter for nonexistent object; try recomputing encounters?");
            match encounter.get_encounter_type() {
                EncounterType::Entrance => {
                    let new_parent = self.objects.iter()
                        .find(|object| object.borrow().get_name() == encounter.get_new_parent())
                        .cloned()
                        .expect("Entrance encounter with nonexistent new parent; try recomputing encounters?");

                    let (new_position, new_velocity) = get_new_position_velocity_entrance(object.clone(), new_parent.clone(), encounter);
                    object.borrow_mut().change_parent(new_parent.clone(), new_position, new_velocity, encounter.get_time());
                }
                EncounterType::Exit => {
                    let new_parent = self.objects.iter()
                        .find(|object| object.borrow().get_name() == encounter.get_new_parent())
                        .cloned()
                        .expect("Exit encounter with nonexistent new parent; try recomputing encounters?");

                    let (new_position, new_velocity) = get_new_position_velocity_exit(object.clone(), encounter);
                    object.borrow_mut().change_parent(new_parent.clone(), new_position, new_velocity, encounter.get_time());
                }
            }
        }

        for object in &self.objects {
            if let Some(orbits) = object.borrow_mut().get_orbits_mut() {
                orbits.front_mut().unwrap().end_at(self.metadata.get_end_time());
            }
        }
    }
}