use std::f64::consts::PI;

use nalgebra_glm::{vec2, DVec2};

use crate::object::orbit::{conic_type::ConicType, orbit_direction::OrbitDirection, orbit_point::OrbitPoint, scary_math::{argument_of_periapsis, kepler_ellipse::EllipseSolver, period, specific_angular_momentum}};

use super::Conic;

pub fn normalize_angle(mut theta: f64) -> f64 {
    theta %= 2.0 * PI;
    (theta + 2.0 * PI) % (2.0 * PI)
}

#[derive(Debug)]
pub struct Ellipse {
    semi_major_axis: f64,
    eccentricity: f64,
    direction: OrbitDirection,
    period: f64,
    argument_of_periapsis: f64,
    specific_angular_momentum: f64,
    solver: EllipseSolver,
}

impl Ellipse {
    pub(in super) fn new(position: DVec2, velocity: DVec2, standard_gravitational_parameter: f64, semi_major_axis: f64, eccentricity: f64, direction: OrbitDirection) -> Self {
        let period = period(standard_gravitational_parameter, semi_major_axis);
        let argument_of_periapsis = argument_of_periapsis(position, velocity, standard_gravitational_parameter);
        let specific_angular_momentum = specific_angular_momentum(position, velocity);
        let solver = EllipseSolver::new(eccentricity);
        Ellipse { semi_major_axis, eccentricity, period, argument_of_periapsis, direction, specific_angular_momentum, solver }
    }
}

impl Conic for Ellipse {
    fn get_theta_from_time_since_periapsis(&self, time_since_periapsis: f64) -> f64 {
        let time_since_periapsis = time_since_periapsis % self.period;
        let mean_anomaly = 2.0 * PI * time_since_periapsis / self.period;
        let eccentric_anomaly = self.solver.solve(mean_anomaly);
        let mut true_anomaly = 2.0 * f64::atan(f64::sqrt((1.0 + self.eccentricity) / (1.0 - self.eccentricity)) * f64::tan(eccentric_anomaly / 2.0));
        // The sign of atan flips halfway through the orbit
        // So we need to add 2pi halfway through the orbit to keep things consistent
        if let OrbitDirection::Clockwise = self.direction {
            true_anomaly = -true_anomaly;
        }
        let theta = true_anomaly + self.argument_of_periapsis;
        let theta = theta % (2.0 * PI);
        if theta < 0.0 {
            theta + 2.0 * PI
        } else {
            theta
        }
    }
    /// Always returns a positive time
    fn get_time_since_last_periapsis(&self, theta: f64) -> f64 {
        let true_anomaly = theta - self.argument_of_periapsis;
        let eccentric_anomaly = 2.0 * f64::atan(f64::sqrt((1.0 - self.eccentricity) / (1.0 + self.eccentricity)) * f64::tan(true_anomaly / 2.0));
        let mut mean_anomaly = eccentric_anomaly - self.eccentricity * f64::sin(eccentric_anomaly);
        if let OrbitDirection::Clockwise = self.direction {
            mean_anomaly = -mean_anomaly;
        }
        mean_anomaly = normalize_angle(mean_anomaly);
        mean_anomaly * self.period / (2.0 * PI)
    }

    fn get_position(&self, theta: f64) -> DVec2 {
        let true_anomaly = theta - self.argument_of_periapsis;
        let radius = (self.semi_major_axis * (1.0 - self.eccentricity.powi(2))) / (1.0 + self.eccentricity * true_anomaly.cos());
        vec2(radius * theta.cos(), radius * theta.sin())
    }
    
    fn get_velocity(&self, position: DVec2, theta: f64) -> DVec2 {
        let true_anomaly = theta - self.argument_of_periapsis;
        let radius = position.magnitude();
        let radius_derivative_with_respect_to_theta = self.semi_major_axis * self.eccentricity * (1.0 - self.eccentricity.powi(2)) * true_anomaly.sin()
            / (self.eccentricity * true_anomaly.cos() + 1.0).powi(2);
        let position_derivative_with_respect_to_theta = vec2(
            radius_derivative_with_respect_to_theta * theta.cos() - radius * theta.sin(), 
            radius_derivative_with_respect_to_theta * theta.sin() + radius * theta.cos());
        let angular_speed = self.specific_angular_momentum / radius.powi(2);
        position_derivative_with_respect_to_theta * angular_speed
    }

    fn get_type(&self) -> ConicType {
        ConicType::Ellipse
    }

    fn get_direction(&self) -> OrbitDirection {
        self.direction
    }

    fn get_period(&self) -> Option<f64> {
        Some(self.period)
    }

    fn get_semi_major_axis(&self) -> f64 {
        self.semi_major_axis
    }

    fn get_semi_minor_axis(&self) -> f64 {
        self.semi_major_axis * f64::sqrt(1.0 - self.eccentricity.powi(2))
    }

    fn get_argument_of_periapsis(&self) -> f64 {
        self.argument_of_periapsis
    }

    fn get_eccentricity(&self) -> f64 {
        self.eccentricity
    }

    fn get_orbits(&self, time: f64) -> i32 {
        (time / self.period) as i32
    }

    fn is_time_between_points(&self, start: &OrbitPoint, end: &OrbitPoint, time: f64) -> bool {
        time > start.get_time() && time < end.get_time()
    }
}
