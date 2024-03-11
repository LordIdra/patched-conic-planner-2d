use std::f64::consts::PI;

fn laguerre_delta(f: f64, f_prime: f64, f_prime_prime: f64) -> f64 {
    let n: f64 = 5.0;
    let mut a = f64::sqrt((n-1.0).powi(2) * f_prime.powi(2) - n*(n-1.0)*f*f_prime_prime);
    a = a.abs() * f_prime.signum();
    - (n*f) / (f_prime + a)
}

fn laguerre_iteration(mean_anomaly: f64, eccentricity: f64, eccentric_anomaly: f64) -> f64 {
    let sin_eccentric_anomaly = eccentric_anomaly.sin();
    let cos_eccentric_anomaly = eccentric_anomaly.cos();
    let f = mean_anomaly - eccentric_anomaly + eccentricity*sin_eccentric_anomaly;
    let f_prime = -1.0 + eccentricity*cos_eccentric_anomaly;
    let f_prime_prime = -eccentricity*sin_eccentric_anomaly;
    eccentric_anomaly + laguerre_delta(f, f_prime, f_prime_prime)
}

/// This is already tested in the conic tests
pub fn solve_kepler_equation_ellipse(eccentricity: f64, mean_anomaly: f64) -> f64 {
    // Choosing an initial seed: https://www.aanda.org/articles/aa/full_html/2022/02/aa41423-21/aa41423-21.html#S5
    // Yes, they're actually serious about that 0.999999 thing (lmao)
    let mut eccentric_anomaly = mean_anomaly
        + (0.999999 * 4.0 * eccentricity * mean_anomaly * (PI - mean_anomaly))
        / (8.0 * eccentricity * mean_anomaly + 4.0 * eccentricity * (eccentricity - PI) + PI.powi(2));

    // Iteration using laguerre method
    // According to this 1985 paper laguerre should practially always converge (they tested it 500,000 times on different values)
    // Also the number of iterations is fixed which is really nice because we can omit a branch every iteration :)
    // https://link.springer.com/content/pdf/10.1007/bf01230852.pdf
    for _ in 0..5 {
        eccentric_anomaly = laguerre_iteration(mean_anomaly, eccentricity, eccentric_anomaly);
    }
    eccentric_anomaly
}