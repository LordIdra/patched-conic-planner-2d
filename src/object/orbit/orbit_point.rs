use nalgebra_glm::DVec2;

use crate::util::format_time;

use super::conic::Conic;

#[derive(Debug, Clone)]
pub struct OrbitPoint {
    theta: f64,
    time: f64,
    time_since_periapsis: f64,
    position: DVec2,
    velocity: DVec2,
}

impl OrbitPoint {
    pub fn new(conic: &dyn Conic, position: DVec2, time: f64) -> Self {
        let theta = f64::atan2(position.y, position.x);
        let time_since_periapsis = conic.get_time_since_last_periapsis(theta);
        println!("{}", format_time(time_since_periapsis));
        let velocity = conic.get_velocity(position, theta);
        Self { theta, time, time_since_periapsis, position, velocity }
    }

    pub fn next(&self, conic: &dyn Conic, delta_time: f64) -> Self {
        let time = self.time + delta_time;
        let mut time_since_periapsis = self.time_since_periapsis + delta_time;
        if let Some(period) = conic.get_period() {
            if time_since_periapsis > period {
                time_since_periapsis -= period;
            }
        }
        let theta = conic.get_theta_from_time_since_periapsis(time_since_periapsis);
        let position = conic.get_position(theta);
        let velocity = conic.get_velocity(position, theta);
        Self { theta, time, time_since_periapsis, position, velocity }
    }

    pub fn get_theta(&self) -> f64 {
        self.theta
    }

    pub fn get_position(&self) -> DVec2 {
        self.position
    }

    pub fn get_velocity(&self) -> DVec2 {
        self.velocity
    }

    pub fn get_time(&self) -> f64 {
        self.time
    }

    pub fn get_time_since_periapsis(&self) -> f64 {
        self.time_since_periapsis
    }

    pub fn is_after(&self, other: &OrbitPoint) -> bool {
        self.time > other.time
    }
}