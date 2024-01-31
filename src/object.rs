use std::{cell::RefCell, rc::Rc, collections::VecDeque};

use nalgebra_glm::DVec2;

use crate::state::SIMULATION_END_TIME;

use self::orbit::Orbit;

pub mod orbit;

enum PhysicsType {
    Stationary(DVec2),
    Orbit(VecDeque<Orbit>),
}

pub struct Object {
    name: String,
    mass: f64,
    physics_type: PhysicsType,
}

impl Object {
    pub fn new_stationary(name: String, mass: f64, position: DVec2) -> Self {
        let physics_type = PhysicsType::Stationary(position);
        Self { name, mass, physics_type }
    }

    pub fn new_orbit(name: String, mass: f64, position: DVec2, velocity: DVec2, parent: Rc<RefCell<Object>>) -> Self {
        let parent_mass = parent.borrow().get_mass();
        let mut orbit = Orbit::new(parent, parent_mass, position, velocity, 0.0);
        orbit.end_at(SIMULATION_END_TIME);
        let mut orbits = VecDeque::new();
        orbits.push_back(orbit);
        let physics_type = PhysicsType::Orbit(orbits);
        Self { name, mass, physics_type }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_mass(&self) -> f64 {
        self.mass
    }

    pub fn get_absolute_position(&self) -> DVec2 {
        match &self.physics_type {
            PhysicsType::Stationary(position) => position.clone(),
            PhysicsType::Orbit(orbits) => orbits.front().unwrap().get_current_point().get_position(),
        }
    }

    pub fn get_orbits(&self) -> Option<&VecDeque<Orbit>> {
        match &self.physics_type {
            PhysicsType::Orbit(orbits) => Some(orbits),
            PhysicsType::Stationary(_) => None,
        }
    }

    pub fn get_orbits_mut(&mut self) -> Option<&mut VecDeque<Orbit>> {
        match &mut self.physics_type {
            PhysicsType::Orbit(orbits) => Some(orbits),
            PhysicsType::Stationary(_) => None,
        }
    }
}