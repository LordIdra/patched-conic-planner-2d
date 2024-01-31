use std::{f64::consts::PI, rc::Rc, cell::RefCell};

use nalgebra_glm::DVec2;

use crate::util::normalize_angle;

use self::{orbit_point::OrbitPoint, orbit_direction::OrbitDirection, conic::{Conic, new_conic}, conic_type::ConicType};

use super::Object;

mod conic_type;
mod conic;
pub mod orbit_direction;
mod orbit_point;
mod scary_math;

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

    //TODO - add this to main project
    pub fn get_remaining_angle(&self) -> f64 {
        // If we have any full orbits remaining, only return up to 2pi
        if self.get_remaining_orbits() > 0 {
            return 2.0 * PI;
        }

        let mut end_theta = normalize_angle(self.end_point.get_theta());
        let current_theta = normalize_angle(self.current_point.get_theta());
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

    pub fn get_completed_orbits(&self) -> i32 {
        self.conic.get_orbits(self.current_point.get_time() - self.start_point.get_time())
    }

    pub fn get_conic_type(&self) -> ConicType {
        self.conic.get_type()
    }

    pub fn get_parent(&self) -> Rc<RefCell<Object>> {
        self.parent.clone()
    }

    pub fn get_semi_major_axis(&self) -> f64 {
        self.conic.get_semi_major_axis()
    }

    pub fn get_semi_minor_axis(&self) -> f64 {
        self.conic.get_semi_minor_axis()
    }

    pub fn get_eccentricity(&self) -> f64 {
        self.conic.get_eccentricity()
    }

    pub fn get_argument_of_periapsis(&self) -> f64 {
        self.conic.get_argument_of_periapsis()
    }

    pub fn get_direction(&self) -> OrbitDirection {
        self.conic.get_direction()
    }

    pub fn get_period(&self) -> Option<f64> {
        self.conic.get_period()
    }

    pub fn is_finished(&self) -> bool {
        self.current_point.is_after(&self.end_point)
    }

    pub fn is_time_within_orbit(&self, time: f64) -> bool {
        self.conic.is_time_between_points(&self.current_point, &self.end_point, time)
    }

    pub fn get_overshot_time(&self, time: f64) -> f64 {
        time - self.end_point.get_time()
    }

    pub fn get_time_since_first_periapsis(&self, theta: f64) -> f64 {
        let mut x = self.get_time_since_last_periapsis(theta);
        if let Some(period) = self.get_period() {
            x += period * self.get_completed_orbits() as f64;
        }
        x
    }

    pub fn get_time_since_last_periapsis(&self, theta: f64) -> f64 {
        self.conic.get_time_since_last_periapsis(theta)
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

    pub fn reset(&mut self) {
        self.current_point = self.start_point.clone();
    }

    pub fn update(&mut self, delta_time: f64) {
        self.current_point = self.current_point.next(&*self.conic, delta_time);
    }
}
