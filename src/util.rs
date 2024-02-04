use std::f64::consts::PI;

pub fn format_time(time: f64) -> String {
    let start_string = if time.is_sign_positive() { "".to_string() } else { "-".to_string() };
    let time = time.abs();
    let years_quotient = f64::floor(time / (360.0 * 24.0 * 60.0 * 60.0));
    let years_remainder = time % (360.0 * 24.0 * 60.0 * 60.0);
    let days_quotient = f64::floor(years_remainder / (24.0 * 60.0 * 60.0));
    let days_remainder = years_remainder % (24.0 * 60.0 * 60.0);
    let hours_quotient = f64::floor(days_remainder / (60.0 * 60.0));
    let hours_remainder = days_remainder % (60.0 * 60.0);
    let minutes_quotient = f64::floor(hours_remainder / 60.0);
    let seconds = f64::round(hours_remainder % 60.0);
    if years_quotient != 0.0 {
        start_string
            + years_quotient.to_string().as_str() + "y"
            + days_quotient.to_string().as_str() + "d"
            + hours_quotient.to_string().as_str() + "h"
            + minutes_quotient.to_string().as_str() + "m"
            + seconds.to_string().as_str() + "s"
    } else if days_quotient != 0.0 {
        start_string
            + days_quotient.to_string().as_str() + "d"
            + hours_quotient.to_string().as_str() + "h"
            + minutes_quotient.to_string().as_str() + "m"
            + seconds.to_string().as_str() + "s"
    } else if hours_quotient != 0.0 {
        start_string
            + hours_quotient.to_string().as_str() + "h"
            + minutes_quotient.to_string().as_str() + "m"
            + seconds.to_string().as_str() + "s"
    } else if minutes_quotient != 0.0 {
        start_string
            + minutes_quotient.to_string().as_str() + "m"
            + seconds.to_string().as_str() + "s"
    } else {
        start_string
            + seconds.to_string().as_str() + "s"
    }
}

pub fn normalize_angle_0_to_2pi(mut theta: f64) -> f64 {
    theta = theta % (2.0 * PI);
    (theta + 2.0 * PI) % (2.0 * PI)
}