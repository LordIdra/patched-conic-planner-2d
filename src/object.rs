use std::{cell::RefCell, rc::Rc, collections::VecDeque};

use nalgebra_glm::DVec2;

use self::orbit::Orbit;

pub mod orbit;

#[derive(Debug)]
enum PhysicsType {
    Stationary(DVec2),
    Orbit(VecDeque<Orbit>),
}

#[derive(Debug)]
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

    pub fn new_orbit(name: String, mass: f64, position: DVec2, velocity: DVec2, parent: Rc<RefCell<Object>>, end_time: f64) -> Self {
        let parent_mass = parent.borrow().get_mass();
        let mut orbit = Orbit::new(parent, parent_mass, position, velocity, 0.0);
        orbit.end_at(end_time);
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

    pub fn get_current_parent(&self) -> Option<Rc<RefCell<Object>>> {
        match &self.physics_type {
            PhysicsType::Stationary(_) => None,
            PhysicsType::Orbit(orbits) => {
                Some(orbits.back().unwrap().get_parent())
            }
        }
    }

    pub fn get_final_parent(&self) -> Option<Rc<RefCell<Object>>> {
        match &self.physics_type {
            PhysicsType::Stationary(_) => None,
            PhysicsType::Orbit(orbits) => {
                Some(orbits.front().unwrap().get_parent())
            }
        }
    }

    pub fn get_soi(&self) -> Option<f64> {
        let orbit = self.get_orbits()?.back().unwrap();
        Some(orbit.get_semi_major_axis() * (self.mass / orbit.get_parent().borrow().get_mass()).powf(2.0 / 5.0))
    }

    pub fn get_current_position(&self) -> DVec2 {
        match &self.physics_type {
            PhysicsType::Stationary(position) => position.clone(),
            PhysicsType::Orbit(orbits) => orbits.back().unwrap().get_current_point().get_position(),
        }
    }

    pub fn get_end_position(&self) -> DVec2 {
        match &self.physics_type {
            PhysicsType::Stationary(position) => position.clone(),
            PhysicsType::Orbit(orbits) => orbits.front().unwrap().get_end_point().get_position(),
        }
    }

    pub fn get_next_position(&self, time_step: f64) -> DVec2 {
        match &self.physics_type {
            PhysicsType::Stationary(position) => position.clone(),
            PhysicsType::Orbit(orbits) => orbits.front().unwrap().get_next_point(time_step).get_position(),
        }
    }

    pub fn get_end_velocity(&self) -> DVec2 {
        match &self.physics_type {
            PhysicsType::Stationary(_) => DVec2::new(0.0, 0.0),
            PhysicsType::Orbit(orbits) => orbits.front().unwrap().get_end_point().get_velocity(),
        }
    }

    pub fn get_next_velocity(&self, time_step: f64) -> DVec2 {
        match &self.physics_type {
            PhysicsType::Stationary(position) => position.clone(),
            PhysicsType::Orbit(orbits) => orbits.front().unwrap().get_next_point(time_step).get_velocity(),
        }
    }

    pub fn get_current_absolute_position(&self) -> DVec2 {
        match &self.physics_type {
            PhysicsType::Stationary(position) => position.clone(),
            PhysicsType::Orbit(_) => self.get_current_parent().unwrap().borrow().get_current_absolute_position() + self.get_current_position(),
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

    pub fn change_parent(&mut self, new_parent: Rc<RefCell<Object>>, new_position: DVec2, new_velocity: DVec2, time: f64) {
        match &mut self.physics_type {
            PhysicsType::Orbit(orbits) => {
                orbits.front_mut().unwrap().end_at(time);
                orbits.push_front(Orbit::new(new_parent.clone(), new_parent.borrow().get_mass(), new_position, new_velocity, time));
            }
            PhysicsType::Stationary(_) => panic!("Cannot change the parent of a stationary object")
        }
    }

    pub fn update_back(&mut self, delta_time: f64) {
        if let Some(orbits) = self.get_orbits_mut() {
            orbits.back_mut().unwrap().update(delta_time);
            if orbits.back().unwrap().is_finished() {
                let previous_end_time = orbits.back().unwrap().get_current_point().get_time();
                orbits.pop_back();
                let overshot_time = previous_end_time - orbits.back().unwrap().get_current_point().get_time();
                orbits.back_mut().unwrap().update(overshot_time);
            }
        }
    }

    pub fn update_front(&mut self, delta_time: f64) {
        if let Some(orbits) = self.get_orbits_mut() {
            orbits.front_mut().unwrap().update(delta_time);
        }
    }
}