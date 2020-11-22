use log::{debug};

use crate::polygon;

/// Star position in image
#[derive(Clone)]
pub struct ImageStar {
    pub pixel_x: f64,
    pub pixel_y: f64,
    pub magnitude: f64
}

/// Image metadata
pub struct Image {
    pub height: u32,
    pub width: u32,
    pub scale_ppa: f64, // Pixels per arcsecond
    pub star_list: Vec<ImageStar>
}

/// Converts the image::Star structure to polygon::Star
pub fn image_star_to_polygon(star_list: &Vec<ImageStar>, scale_app: f64) -> Vec<polygon::Star> {
    let scale_rad = (scale_app / 60.0 / 60.0).to_radians();
    let mut pol_star_list = Vec::new();
    debug!("Image Star to Polygon > Star list:{} Scale \"pp:{} Scale rpp:{}", star_list.len(), scale_app, scale_rad);
    for (i, star) in star_list.iter().enumerate() {
        let polygon_star = polygon::Star {
            id: i as u64,
            db_id: 0,   // No catalogue reference
            ra: 0.0,  // Right Ascension unknown
            dec: 0.0, // Declination unknown
            ra_rad: star.pixel_x * scale_rad,   // Relative RA
            dec_rad: star.pixel_y * scale_rad,  // Relative Dec
            magnitude: star.magnitude
        };
        debug!(" i:{} x:{} y:{} ra_rad:{} dec_rad:{}",
                i,
                star.pixel_x,
                star.pixel_y,
                polygon_star.ra_rad,
                polygon_star.dec_rad
            );
        pol_star_list.push(polygon_star);
    }
    pol_star_list
}