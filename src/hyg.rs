use std::error::Error;
use log::{debug};

use csv;

use crate::math;
use crate::polygon;

/// Reads HYG star database CSV file to memory
pub fn read_stars_from_file(ra_center: f64, dec_center: f64, radii: f64, magnitude_limit: f64) -> Result<Vec<polygon::Star>, Box<dyn Error>> {
    let ra_center_rad = math::hours_to_radians(ra_center);
    let dec_center_rad = math::degrees_to_radians(dec_center.to_radians());
    let radii_rad = math::degrees_to_radians(radii);

    // Read database
    let mut star_list: Vec<polygon::Star> = Vec::new();
    let mut reader = csv::Reader::from_path("hygfull-compact.csv")?;
    let headers = reader.headers()?;
    debug!("{:?}", headers);
    for row in reader.records() {
        // Initialize record
        let mut star = polygon::Star {
            id: 0,
            hip: 0,
            ra: 0.0,
            dec: 0.0,
            ra_rad: 0.0,
            dec_rad: 0.0,
            magnitude: 0.0,
        };
        // debug!("Row: {:?}", row);
        let record = row?;
        // Read record data
        star.id = record.get(0).unwrap().parse::<u32>().unwrap();
        star.hip = record.get(1).unwrap().parse::<u32>().unwrap();
        star.ra = record.get(2).unwrap().parse::<f64>().unwrap();
        star.dec = record.get(3).unwrap().parse::<f64>().unwrap();
        star.magnitude = record.get(4).unwrap().parse::<f64>().unwrap();
        // Transform degrees/hours to radians
        star.ra_rad = math::hours_to_radians(star.ra);
        star.dec_rad = math::degrees_to_radians(star.dec);

        // Calculate angular separation between star and center
        let sep_rad = math::angular_separation_radians(ra_center_rad, dec_center_rad, star.ra_rad, star.dec_rad);
        // Filter by magnitude and angular separation
        if star.magnitude < magnitude_limit && sep_rad <= radii_rad {
            // Add star to the list
            star_list.push(star);
        }
    }
    Ok(star_list)
}
