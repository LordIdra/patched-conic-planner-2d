use std::fmt::Debug;

use nalgebra_glm::DVec2;

use crate::constants::GRAVITATIONAL_CONSTANT;

use self::{ellipse::Ellipse, hyperbola::Hyperbola};

use super::{orbit_direction::OrbitDirection, orbit_point::OrbitPoint, scary_math::{semi_major_axis, eccentricity}, conic_type::ConicType};

mod ellipse;
mod hyperbola;
pub fn new_conic(parent_mass: f64, position: DVec2, velocity: DVec2) -> Box<dyn Conic> {
    let standard_gravitational_parameter = GRAVITATIONAL_CONSTANT * parent_mass;
    let semi_major_axis = semi_major_axis(position, velocity, standard_gravitational_parameter);
    let eccentricity = eccentricity(position, velocity, standard_gravitational_parameter, semi_major_axis);
    let direction = OrbitDirection::new(position, velocity);
    if eccentricity <= 1.0 {
        Box::new(Ellipse::new(position, velocity, standard_gravitational_parameter, semi_major_axis, eccentricity, direction))
    } else {
        Box::new(Hyperbola::new(position, velocity, standard_gravitational_parameter, semi_major_axis, eccentricity, direction))
    }
}

/// Describes all the static parmeters of an orbit, but says nothing about the current state of the object in the orbit
pub trait Conic: Debug + Send {
    fn get_theta_from_time_since_periapsis(&self, time: f64) -> f64;
    fn get_time_since_last_periapsis(&self, theta: f64) -> f64;
    fn get_type(&self) -> ConicType;
    fn get_position(&self, theta: f64) -> DVec2;
    fn get_velocity(&self, position: DVec2, theta: f64) -> DVec2;
    fn get_direction(&self) -> OrbitDirection;
    fn get_period(&self) -> Option<f64>;
    fn get_semi_major_axis(&self) -> f64;
    fn get_semi_minor_axis(&self) -> f64;
    fn get_argument_of_periapsis(&self) -> f64;
    fn get_eccentricity(&self) -> f64;
    fn get_orbits(&self, remaining_time: f64) -> i32;
    fn is_time_between_points(&self, start: &OrbitPoint, end: &OrbitPoint, time_since_periapsis: f64) -> bool;
}
