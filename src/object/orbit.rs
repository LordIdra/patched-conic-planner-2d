use std::{f64::consts::PI, rc::Rc, cell::RefCell};

use nalgebra_glm::DVec2;

use crate::util::normalize_angle_0_to_2pi;

use self::{orbit_point::OrbitPoint, orbit_direction::OrbitDirection, conic::{Conic, new_conic}};

use super::Object;

mod conic_type;
mod conic;
pub mod orbit_direction;
mod orbit_point;
mod scary_math;

#[derive(Debug)]
pub struct Orbit {
    parent: Rc<RefCell<Object>>,
    conic: Box<dyn Conic>,
    start_point: OrbitPoint,
    end_point: OrbitPoint,
    current_point: OrbitPoint,
}

impl Orbit {
    pub fn new(parent: Rc<RefCell<Object>>, parent_mass: f64, position: DVec2, velocity: DVec2, time: f64) -> Self {
        let conic = new_conic(parent_mass, position, velocity);
        let start_point = OrbitPoint::new(&*conic, position, time);
        let end_point = start_point.clone();
        let current_point = start_point.clone();
        Self { parent, conic, start_point, end_point, current_point }
    }

    pub fn get_start_point(&self) -> &OrbitPoint {
        &self.start_point
    }

    pub fn get_current_point(&self) -> &OrbitPoint {
        &self.current_point
    }

    pub fn get_end_point(&self) -> &OrbitPoint {
        &self.end_point
    }

    pub fn get_remaining_angle(&self) -> f64 {
        // If we have any full orbits remaining, only return up to 2pi
        if self.get_remaining_orbits() > 0 {
            return 2.0 * PI;
        }

        let mut end_theta = normalize_angle_0_to_2pi(self.end_point.get_theta());
        let current_theta = normalize_angle_0_to_2pi(self.current_point.get_theta());
        if let OrbitDirection::AntiClockwise = self.conic.get_direction() {
            if end_theta < current_theta {
                end_theta += 2.0 * PI;
            }
        } else {
            if end_theta > current_theta {
                end_theta -= 2.0 * PI;
            }
        }
        end_theta - current_theta
    }

    pub fn get_remaining_orbits(&self) -> i32 {
        self.conic.get_orbits(self.end_point.get_time() - self.current_point.get_time())
    }

    pub fn get_parent(&self) -> Rc<RefCell<Object>> {
        self.parent.clone()
    }

    pub fn get_semi_major_axis(&self) -> f64 {
        self.conic.get_semi_major_axis()
    }

    pub fn is_finished(&self) -> bool {
        self.current_point.is_after(&self.end_point)
    }

    pub fn get_first_periapsis_time(&self) -> f64 {
        self.start_point.get_time() - self.start_point.get_time_since_periapsis()
    }

    pub fn get_theta_from_time(&self, time: f64) -> f64 {
        let time_since_periapsis = time - self.get_first_periapsis_time();
        self.conic.get_theta_from_time_since_periapsis(time_since_periapsis)
    }

    pub fn get_position_from_theta(&self, theta: f64) -> DVec2 {
        self.conic.get_position(theta)
    }

    pub fn get_velocity_from_theta(&self, theta: f64) -> DVec2 {
        self.conic.get_velocity(self.get_position_from_theta(theta), theta)
    }

    pub fn end_at(&mut self, time: f64) {
        let theta = self.get_theta_from_time(time);
        let position = self.get_position_from_theta(theta);
        self.end_point = OrbitPoint::new(&*self.conic, position, time);
    }

    pub fn update(&mut self, delta_time: f64) {
        self.current_point = self.current_point.next(&*self.conic, delta_time);
    }
}
