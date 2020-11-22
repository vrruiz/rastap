///  Hours (right ascension) to radians
pub fn hours_to_radians(hours: f64) -> f64 {
    hours * 0.26179938779914943653855361527329
}

/// Calculate angular separation (Source: Astronomical Algorithms, Meeus)
pub fn angular_separation_radians(ra1: f64, dec1: f64, ra2: f64, dec2: f64) -> f64 {   
    // cos(d) = sin(d1) * sin(d2) + cos(d1) * cos(d2) * cos(a1 - a2)
    (dec1.sin() * dec2.sin() + dec1.cos() * dec2.cos() * (ra2 - ra1).cos()).acos()
}
