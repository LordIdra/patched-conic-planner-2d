use std::f64::consts::PI;

const DELTA_THRESHOLD: f64 = 1.0e-10;

pub fn laguerre_delta(f: f64, f_prime: f64, f_prime_prime: f64) -> f64 {
    let n: f64 = 2.0; // N=2, modified Newton-Raphson
    let a = (n-1.0).powi(2) * f_prime.powi(2) - n*(n-1.0)*f*f_prime_prime;
    let mut b = f64::sqrt(a.abs());
    b = b.abs() * f_prime.signum(); // prevent catastrophic cancellation
    - (n*f) / (f_prime + b)
}

#[derive(Debug)]
pub struct EllipseSolver {
    eccentricity: f64
}

impl EllipseSolver {
    pub fn new(eccentricity: f64) -> Self {
        Self { eccentricity }
    }

    /// Assumes 0 < mean_anomaly < 2pi
    pub fn solve(&self, mean_anomaly: f64) -> f64 {
        // Choosing an initial seed: https://www.aanda.org/articles/aa/full_html/2022/02/aa41423-21/aa41423-21.html#S5
        // Yes, they're actually serious about that 0.999999 thing (lmao)
        let mut eccentric_anomaly = mean_anomaly
            + (0.999999 * 4.0 * self.eccentricity * mean_anomaly * (PI - mean_anomaly))
            / (8.0 * self.eccentricity * mean_anomaly + 4.0 * self.eccentricity * (self.eccentricity - PI) + PI.powi(2));

        // Iteration using laguerre method
        // According to this 1985 paper laguerre should practially always converge (they tested it 500,000 times on different values)
        // https://link.springer.com/content/pdf/10.1007/bf01230852.pdf
        loop {
            let sin_eccentric_anomaly = eccentric_anomaly.sin();
            let cos_eccentric_anomaly = eccentric_anomaly.cos();
            let f = mean_anomaly - eccentric_anomaly + self.eccentricity*sin_eccentric_anomaly;
            let f_prime = -1.0 + self.eccentricity*cos_eccentric_anomaly;
            let f_prime_prime = -self.eccentricity*sin_eccentric_anomaly;
            let delta = laguerre_delta(f, f_prime, f_prime_prime);
            if delta.abs() < DELTA_THRESHOLD {
                break;
            }
            eccentric_anomaly = eccentric_anomaly + delta;
        }
        eccentric_anomaly
    }
}