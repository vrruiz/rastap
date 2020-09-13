use std::{
    error::Error,
    path::Path
};

use csv;
use log::{debug};

use crate::image::{ImageStar};

/// Reads a sextractor result file, converted to CSV
pub fn read_image_stars_from_file(path: &Path) -> Result<Vec<ImageStar>, Box<dyn Error>> {
    // Read database
    let mut star_list: Vec<ImageStar> = Vec::new();
    let mut reader = csv::Reader::from_path(path)?;
    let headers = reader.headers()?;
    debug!("Read sextractor > Headers > {:?}", headers); 
    for row in reader.records() {
        debug!("  Row: {:?}", row);
        // Initialize star record
        let mut star = ImageStar {
            pixel_x: 0.0,
            pixel_y: 0.0,
            magnitude: 0.0,
        };
        let record = row?;
        // Read record data
        star.pixel_x = record.get(0).unwrap().parse::<f32>().unwrap();
        star.pixel_y = record.get(1).unwrap().parse::<f32>().unwrap();
        star.magnitude = record.get(2).unwrap().parse::<f32>().unwrap();
        // Sort by magnitude
        star_list.sort_by(|a, b| b.magnitude.partial_cmp(&a.magnitude).unwrap());
        star_list.push(star);
    }
    Ok(star_list)
}
