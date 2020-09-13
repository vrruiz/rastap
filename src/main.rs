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

#[derive(Debug, StructOpt)]
#[structopt(about)]
struct Cli {
    /// Path to sextractor file.
    #[structopt(name = "sex-file", parse(from_os_str))]
    sex_file: PathBuf,
}

impl Cli {
    /// Gets the path to the input XISF file.
    pub fn sex_file(&self) -> &Path {
        self.sex_file.as_path()
    }
}

fn main() -> io::Result<()> {
    // Init logger
    env_logger::builder().format_timestamp(None).init();
 
    // CLI interface information.
    let cli = Cli::from_args();

    // Read star database (HYG) file
    let mut star_list: Vec<polygon::Star> = Vec::new();
    match hyg::read_stars_from_file(0.0, 0.0, 5.0, 10.0) {
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
    match sextractor::read_image_stars_from_file(cli.sex_file()) {
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