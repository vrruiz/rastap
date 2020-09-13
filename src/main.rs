use std::{
    error::Error,
    io,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

use env_logger;
use log::{debug};

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
    ra_deg: f32,

    /// Declination center of search in degrees and decimals (dd.xx)
    #[structopt(long = "dec")]
    dec_deg: f32,

    /// Search radii in degrees and decimals (dd.xx)
    #[structopt(long = "radii")]
    radii_deg: f32,

    /// Limiting magnitud
    #[structopt(long = "male", default_value="10.0")]
    male: f32,

    /// Path to sextractor file.
    #[structopt(long = "sex-csv", parse(from_os_str))]
    sex_csv: PathBuf,

    /// Image scale in pixels per arcsecond
    #[structopt(short,long)]
    scale: f32,
}

impl Cli {
    /// Gets the search center Right Ascension (R.A.)
    pub fn ra_deg(&self) -> f32 {
        self.ra_deg
    }

    /// Gets the search center Declination (Dec)
    pub fn dec_deg(&self) -> f32 {
        self.dec_deg
    }

    /// Gets the search radii (Dec)
    pub fn radii_deg(&self) -> f32 {
        self.radii_deg
    }

    /// Gets the search radii (Dec)
    pub fn male(&self) -> f32 {
        self.male
    }

    /// Gets the path to the input sextractor file.
    pub fn sex_csv(&self) -> &Path {
        self.sex_csv.as_path()
    }

    /// Gets the image scale in pixels per arcsecond.
    pub fn scale(&self) -> f32 {
        self.scale
    }
}

fn main() -> io::Result<()> {
    // Init logger
    env_logger::builder().format_timestamp(None).init();
 
    // CLI interface information.
    let cli = Cli::from_args();

    // Read star database (HYG) file
    let mut star_list: Vec<polygon::Star> = Vec::new();
    match hyg::read_stars_from_file(cli.ra_deg(), cli.dec_deg(), cli.radii_deg(), cli.male()) {
        Ok(star_list_read) => {
            star_list = star_list_read;
        }
        Err(err) => println!("Error {:?}", err),
    }
    for star in &star_list {
        println!("Star id:{}\thip:{}\tra:{} \tdec:{}\tmagnitude:{}", star.id, star.hip, star.ra, star.dec, star.magnitude);
    }
    match polygon::find_polygons(&star_list) {
        Some(polygons) => {
            for polygon in polygons {
                println!("{}-gon for star {}: {:?} {:?}", polygon::POLYGON_EDGES, polygon.star_index, polygon.length_list, polygon.star_list);
            }
        },
        None => println!("None")
    }
    println!("Star list length: {}", star_list.len());

    // Read list
    let mut image_star_list: Vec<image::ImageStar> = Vec::new();
    match sextractor::read_image_stars_from_file(cli.sex_csv()) {
        Ok(image_star_list_read) => {
            for star in &image_star_list_read {
                println!("Image Star x:{} y:{} mag:{}", star.pixel_x, star.pixel_y, star.magnitude);
            }
            let pol_star_list = image::image_star_to_polygon(&image_star_list_read, 1.90);
            for star in &pol_star_list {
                println!("Polygon Star: x:{} y:{} mag:{}", star.ra_rad, star.dec_rad, star.magnitude);
            }
            match polygon::find_polygons(&pol_star_list) {
                Some(image_polygons) => {
                    for pol in image_polygons {
                        println!("{}-gon for star {}: {:?} {:?}", polygon::POLYGON_EDGES, pol.star_index, pol.length_list, pol.star_list);
                    }
                },
                None => println!("Couldn't find polygons in the image")
            }
            image_star_list = image_star_list_read;
        }
        Err(err) => println!("Error reading image star list: {}", err)
    }
    println!("Image list length: {}", star_list.len());
 
    Ok(())
}