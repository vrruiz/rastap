/// Convert degrees (declination) to radians
pub fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * 0.017453292519943295769236907684886
}

/// Convert hours (right ascension) to radians
pub fn hours_to_radians(hours: f32) -> f32 {
    hours * 0.26179938779914943653855361527329
}

/// Calculate angular separation (Source: Astronomical Algorithms, Meeus)
pub fn angular_separation_radians(ra1: f32, dec1: f32, ra2: f32, dec2: f32) -> f32 {   
    // cos(d) = sin(d1) * sin(d2) + cos(d1) * cos(d2) * cos(a1 - a2)
    (dec1.sin() * dec2.sin() + dec1.cos() * dec2.cos() * (ra2 - ra1).cos()).acos()
}