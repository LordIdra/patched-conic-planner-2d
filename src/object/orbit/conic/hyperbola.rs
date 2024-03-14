use std::f64::consts::PI;

use nalgebra_glm::{vec2, DVec2};

use crate::object::orbit::{conic_type::ConicType, orbit_direction::OrbitDirection, orbit_point::OrbitPoint, scary_math::{argument_of_periapsis, kepler_hyperbola::HyperbolaSolver, specific_angular_momentum}};

use super::Conic;

#[derive(Debug)]
pub struct Hyperbola {
    standard_gravitational_parameter: f64,
    semi_major_axis: f64,
    eccentricity: f64,
    direction: OrbitDirection,
    argument_of_periapsis: f64,
    specific_angular_momentum: f64,
    solver: HyperbolaSolver,
}

impl Hyperbola {
    pub(in super) fn new(position: DVec2, velocity: DVec2, standard_gravitational_parameter: f64, semi_major_axis: f64, eccentricity: f64, direction: OrbitDirection) -> Self {
        let argument_of_periapsis = argument_of_periapsis(position, velocity, standard_gravitational_parameter);
        let specific_angular_momentum = specific_angular_momentum(position, velocity);
        let solver = HyperbolaSolver::new(eccentricity);
        Hyperbola { standard_gravitational_parameter, semi_major_axis, eccentricity, argument_of_periapsis, direction, specific_angular_momentum, solver }
    }
}

impl Conic for Hyperbola {
    fn get_theta_from_time_since_periapsis(&self, time_since_periapsis: f64) -> f64 {
        let x = self.standard_gravitational_parameter.powi(2) / self.specific_angular_momentum.powi(3);
        let mean_anomaly = x * time_since_periapsis * (self.eccentricity.powi(2) - 1.0).powf(3.0 / 2.0);
        let eccentric_anomaly = self.solver.solve(mean_anomaly);
        let true_anomaly = 2.0 * f64::atan(f64::sqrt((self.eccentricity + 1.0) / (self.eccentricity - 1.0)) * f64::tanh(eccentric_anomaly / 2.0));
        let theta = true_anomaly + self.argument_of_periapsis;
        let theta = theta % (2.0 * PI);
        if theta < 0.0 {
            theta + 2.0 * PI
        } else {
            theta
        }
    }

    /// Time can be negative if we have not reached the periapsis at the given theta
    fn get_time_since_last_periapsis(&self, theta: f64) -> f64 {
        let true_anomaly = theta - self.argument_of_periapsis;
        let eccentric_anomaly = 2.0 * f64::atanh(f64::sqrt((self.eccentricity - 1.0) / (self.eccentricity + 1.0)) * f64::tan(true_anomaly / 2.0));
        let mean_anomaly = self.eccentricity * f64::sinh(eccentric_anomaly) - eccentric_anomaly;
        let x = self.specific_angular_momentum.powi(3) / self.standard_gravitational_parameter.powi(2);
        mean_anomaly * x / (self.eccentricity.powi(2) - 1.0).powf(3.0 / 2.0)
    }

    fn get_position(&self, theta: f64) -> DVec2 {
        let true_anomaly = theta - self.argument_of_periapsis;
        let radius = (self.semi_major_axis * (1.0 - self.eccentricity.powi(2))) / (1.0 + self.eccentricity * true_anomaly.cos());
        vec2(radius * theta.cos(), radius * theta.sin())
    }
    
    fn get_velocity(&self, position: DVec2, theta: f64) -> DVec2 {
        let true_anomaly = theta - self.argument_of_periapsis;
        let radius = position.magnitude();
        let radius_derivative_with_respect_to_true_anomaly = self.semi_major_axis * self.eccentricity * (1.0 - self.eccentricity.powi(2)) * true_anomaly.sin()
            / (self.eccentricity * true_anomaly.cos() + 1.0).powi(2);
        let position_derivative_with_respect_to_true_anomaly = vec2(
            radius_derivative_with_respect_to_true_anomaly * theta.cos() - radius * theta.sin(), 
            radius_derivative_with_respect_to_true_anomaly * theta.sin() + radius * theta.cos());
        let angular_speed = self.specific_angular_momentum / radius.powi(2);
        position_derivative_with_respect_to_true_anomaly * angular_speed
    }

    fn get_type(&self) -> ConicType {
        ConicType::Hyperbola
    }

    fn get_direction(&self) -> OrbitDirection {
        self.direction
    }

    fn get_period(&self) -> Option<f64> {
        None
    }

    fn get_semi_major_axis(&self) -> f64 {
        self.semi_major_axis
    }

    fn get_semi_minor_axis(&self) -> f64 {
        self.semi_major_axis * f64::sqrt(self.eccentricity.powi(2) - 1.0)
    }

    fn get_argument_of_periapsis(&self) -> f64 {
        self.argument_of_periapsis
    }

    fn get_eccentricity(&self) -> f64 {
        self.eccentricity
    }

    fn get_orbits(&self, _: f64) -> i32 {
        0
    }

    fn is_time_between_points(&self, start: &OrbitPoint, end: &OrbitPoint, time: f64) -> bool {
        time > start.get_time() && time < end.get_time()
    }
}
