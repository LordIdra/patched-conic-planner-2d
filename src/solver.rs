use serde::Deserialize;

use crate::{case::CaseData, constants::SIMULATION_END_TIME};

#[derive(Deserialize)]
pub enum EncounterType {
    Enter,
    Exit,
}

#[derive(Deserialize)]
pub struct Encounter {
    encounter_type: EncounterType,
    object: String,
    new_parent: String,
    time: f64,
}

pub fn solve(case_data: CaseData) -> Vec<Encounter> {
    let objects = case_data.get_objects();
    let time = 0.0;
    let encounters = vec![];
    while time < SIMULATION_END_TIME {
        for object in objects {
            
        }
    }
    encounters
}