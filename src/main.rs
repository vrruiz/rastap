use std::{
    error::Error,
    io,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

use env_logger;
use log::{debug};

mod gaia_db;
mod hyg;
mod image;
mod math;
mod polygon;
mod sextractor;

/// Command line arguments
#[derive(Debug, StructOpt)]
#[structopt(about)]
struct Cli {
    /// Right Ascension center of search in hours and decimals (hh.xx)
    #[structopt(long = "ra")]
    ra_deg: f64,

    /// Declination center of search in degrees and decimals (dd.xx)
    #[structopt(long = "dec")]
    dec_deg: f64,

    /// Search radii in degrees and decimals (dd.xx)
    #[structopt(long = "radii")]
    radii_deg: f64,

    /// Limiting magnitud
    #[structopt(long = "male", default_value="10.0")]
    male: f64,

    /// Path to sextractor file.
    #[structopt(long = "sex-csv", parse(from_os_str))]
    sex_csv: PathBuf,

    /// Image scale in pixels per arcsecond
    #[structopt(short,long)]
    scale: f64,
}

impl Cli {
    /// Gets the search center Right Ascension (R.A.)
    pub fn ra_deg(&self) -> f64 {
        self.ra_deg
    }

    /// Gets the search center Declination (Dec)
    pub fn dec_deg(&self) -> f64 {
        self.dec_deg
    }

    /// Gets the search radii (Dec)
    pub fn radii_deg(&self) -> f64 {
        self.radii_deg
    }

    /// Gets the search radii (Dec)
    pub fn male(&self) -> f64 {
        self.male
    }

    /// Gets the path to the input sextractor file.
    pub fn sex_csv(&self) -> &Path {
        self.sex_csv.as_path()
    }

    /// Gets the image scale in pixels per arcsecond.
    pub fn scale(&self) -> f64 {
        self.scale
    }
}

// Find polygons
fn find_polygons_and_fit(star_list: Vec<polygon::Star>, image_star_list: Vec<image::ImageStar>, scale: f64) {
    let mut star_polygons: Vec<polygon::Polygon> = Vec::new();
    let mut image_polygons: Vec<polygon::Polygon> = Vec::new();

    // Convert list to ImageStar
    let pol_star_list = image::image_star_to_polygon(&image_star_list, scale);
    // Limit list
    // pol_star_list.truncate(image_star_list.len());

    // Find star polygons
    match polygon::find_polygons(&star_list) {
        Some(polygons) => {
            for polygon in &polygons {
                println!("{}-gon for star {}: {:?} {:?}", polygon::POLYGON_EDGES, polygon.star_index, polygon.length_list, polygon.star_list);
            }
            star_polygons = polygons;
        },
        None => println!("None")
    }
    println!("Star list length: {}", star_list.len());
    for star in &pol_star_list {
        println!("Polygon Star: x:{} y:{} mag:{}", star.ra_rad, star.dec_rad, star.magnitude);
    }

    // Find image polygons
    match polygon::find_polygons(&pol_star_list) {
        Some(polygons) => {
            println!("POL,pixel1_x,pixel1_y,pixel2_x,pixel2_y,pixel3_x,pixel3_y,pixel4_x,pixel4_y");
            'finish: for (n, pol) in polygons.iter().enumerate() {
                println!("{}-gon for star {}: {:?} {:?}", polygon::POLYGON_EDGES, pol.star_index, pol.length_list, pol.star_list);
                let mut pol_string = "".to_owned();
                for (i, star) in pol.star_list.iter().enumerate() {
                    if i > 0 {
                        pol_string.push_str(",");
                    }
                    let coordinates = format!("{},{}", image_star_list[*star].pixel_x, image_star_list[*star].pixel_y);
                    pol_string.push_str(&coordinates);
                }
                println!("POL,{}", pol_string);
            }
            image_polygons = polygons;
        },
        None => println!("Couldn't find polygons in the image")
    }

    // Compare star database and image polygons
    println!("Searching similarities");
    polygon::find_fit(&image_polygons, &star_polygons);
}

fn main() -> io::Result<()> {
    // Init logger
    env_logger::builder().format_timestamp(None).init();
 
    // CLI interface information
    let cli = Cli::from_args();

    // Read star database (Mini Gaia DR2) file
    let mut star_list: Vec<polygon::Star> = Vec::new();

    match gaia_db::read_stars_from_file(cli.ra_deg(), cli.dec_deg(), cli.radii_deg(), cli.male()) {
        Ok(star_list_read) => {
            star_list = star_list_read;
        }
        Err(err) => println!("Error {:?}", err),
    }
    for star in &star_list {
        println!("Star id:{}\tdb_id:{}\tra:{} \tdec:{}\tmagnitude:{}", star.id, star.db_id, star.ra, star.dec, star.magnitude);
    }

    // Read star coordinates from sextractor
    let mut image_star_list: Vec<image::ImageStar> = Vec::new();
    match sextractor::read_image_stars_from_file(cli.sex_csv()) {
        Ok(image_star_list_read) => {
            for star in &image_star_list_read {
                println!("Image Star x:{} y:{} mag:{}", star.pixel_x, star.pixel_y, star.magnitude);
            }
            image_star_list = image_star_list_read;
        }
        Err(err) => println!("Error reading image star list: {}", err)
    }
    println!("Image list length: {}", image_star_list.len());

    star_list.truncate(500);
    image_star_list.truncate(500);

    // If stars found on the image, then find and match the polygons
    if image_star_list.len() > 10 {
        find_polygons_and_fit(star_list, image_star_list, cli.scale());
    }
 
    Ok(())
}