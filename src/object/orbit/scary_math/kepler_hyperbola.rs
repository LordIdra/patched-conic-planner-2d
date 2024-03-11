use std::process::exit;

fn laguerre_delta(f: f64, f_prime: f64, f_prime_prime: f64) -> f64 {
    let n: f64 = 5.0;
    let mut a = f64::sqrt((n-1.0).powi(2) * f_prime.powi(2) - n*(n-1.0)*f*f_prime_prime);
    dbg!((n-1.0).powi(2) * f_prime.powi(2) - n*(n-1.0)*f*f_prime_prime);
    a = a.abs() * f_prime.signum();
    - (n*f) / (f_prime + a)
}

fn laguerre_iteration(mean_anomaly: f64, eccentricity: f64, eccentric_anomaly: f64) -> f64 {
    let sinh_eccentric_anomaly = eccentric_anomaly.sinh();
    let cosh_eccentric_anomaly = eccentric_anomaly.cosh();
    let f = mean_anomaly + eccentric_anomaly - eccentricity*sinh_eccentric_anomaly;
    let f_prime = 1.0 - eccentricity*cosh_eccentric_anomaly;
    let f_prime_prime = -eccentricity*sinh_eccentric_anomaly;
    dbg!(f, f_prime, f_prime_prime);
    eccentric_anomaly + laguerre_delta(f, f_prime, f_prime_prime)
}

/// This is already tested in the conic tests
pub fn solve_kepler_equation_hyperbola(eccentricity: f64, mean_anomaly: f64) -> f64 {
    let mut eccentric_anomaly = mean_anomaly.abs();

    // Iteration using laguerre method
    // According to this 1985 paper laguerre should practially always converge (they tested it 500,000 times on different values)
    // Also the number of iterations is fixed which is really nice because we can omit a branch every iteration :)
    // https://link.springer.com/content/pdf/10.1007/bf01230852.pdf
    dbg!(eccentricity, mean_anomaly);
    for _ in 0..5 {
        eccentric_anomaly = laguerre_iteration(mean_anomaly, eccentricity, eccentric_anomaly);
    }
    println!("{}", eccentric_anomaly);
    exit(0);
    eccentric_anomaly * mean_anomaly.signum()
}