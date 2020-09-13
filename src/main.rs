use std::error::Error;

use csv;
use env_logger;
use log::{debug, info};

const POLYGON_EDGES: usize = 4;

/// Polygon structure
struct Polygon {
    star_index: usize,
    star_list: Vec<usize>,
    length_list: Vec<f32>,
    center_ra_rad: f32,
    center_dec_rad: f32,
}

/// Calculate the number of size of the polygon
fn polygon_connections(polygon: usize) -> usize {
    let mut sides = 0;
    for i in 1..polygon {
        sides = sides + polygon - i;
    }
    return sides;
}

/// Star data structure
struct Star {
    id: u32,
    hip: u32,
    ra: f32,
    dec: f32,
    ra_rad: f32,
    dec_rad: f32,
    magnitude: f32
}

/// Convert degrees (declination) to radians
fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * 0.017453292519943295769236907684886
}

/// Convert hours (right ascension) to radians
fn hours_to_radians(hours: f32) -> f32 {
    hours * 0.26179938779914943653855361527329
}

/// Calculate angular separation (Source: Astronomical Algorithms, Meeus)
fn angular_separation_radians(ra1: f32, dec1: f32, ra2: f32, dec2: f32) -> f32 {   
    // cos(d) = sin(d1) * sin(d2) + cos(d1) * cos(d2) * cos(a1 - a2)
    (dec1.sin() * dec2.sin() + dec1.cos() * dec2.cos() * (ra2 - ra1).cos()).acos()
}

/// Calculate star distance between two stars
fn star_distance_rad(star_a: &Star, star_b: &Star) -> f32 {
    ((star_b.ra_rad - star_a.ra_rad).abs()).sqrt() + ((star_b.dec_rad - star_a.dec_rad).abs()).sqrt()
}

/// Reads HYG star database CSV file to memory
fn read_stars_from_file(ra_center: f32, dec_center: f32, radii: f32, magnitude_limit: f32) -> Result<Vec<Star>, Box<dyn Error>> {
    let ra_center_rad = hours_to_radians(ra_center);
    let dec_center_rad = degrees_to_radians(dec_center.to_radians());
    let radii_rad = degrees_to_radians(radii);

    // Read database
    let mut star_list: Vec<Star> = Vec::new();
    let mut reader = csv::Reader::from_path("hygfull-compact.csv")?;
    let headers = reader.headers()?;
    info!("{:?}", headers);
    for row in reader.records() {
        // Initialize record
        let mut star = Star {
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
        star.ra = record.get(2).unwrap().parse::<f32>().unwrap();
        star.dec = record.get(3).unwrap().parse::<f32>().unwrap();
        star.magnitude = record.get(4).unwrap().parse::<f32>().unwrap();
        // Transform degrees/hours to radians
        star.ra_rad = hours_to_radians(star.ra);
        star.dec_rad = degrees_to_radians(star.dec);

        // Calculate angular separation between star and center
        let sep_rad = angular_separation_radians(ra_center_rad, dec_center_rad, star.ra_rad, star.dec_rad);
        // Filter by magnitude and angular separation
        if star.magnitude < magnitude_limit && sep_rad <= radii_rad {
            // Add star to the list
            star_list.push(star);
        }
    }
    Ok(star_list)
}

/// Find polygons. For each star, the POLYGON_EDGES-1 closest stars.
fn find_polygons(star_list: &Vec<Star>) -> Option<Vec<Polygon>> {
    let mut polygons: Vec<Polygon> = Vec::new();
    let conn_number = polygon_connections(POLYGON_EDGES);
    if star_list.len() < POLYGON_EDGES {
        // Not enough stars for the polygon
        return None;
    }
    // For each star find the POLYGON_EDGES - 1 closest stars
    for (id_a, star_a) in star_list.iter().enumerate() {
        debug!("Find polygon > Searching for star i:{} id:({})", id_a, star_a.id);
        let mut star_vec = vec![0_usize; POLYGON_EDGES];
        let mut length_vec = vec![0_f32; conn_number];
        let mut dist_vec = vec![f32::MAX; POLYGON_EDGES];
        for (id_b, star_b) in star_list.iter().enumerate() {
            if id_a != id_b {
                // First vertex of the polygon is the star itself, skip
                // Calculate distance between the stars
                let distance = star_distance_rad(star_a, star_b);
                // Compare this distance with the current list of closest stars
                let length = dist_vec.len();
                let mut finished = false;
                let mut i = 0;
                while i < length && finished == false {
                    if distance < dist_vec[i] {
                        // Star is closer, insert new value
                        star_vec.insert(i, id_b);
                        dist_vec.insert(i, distance);
                        // And discard the last element of the list
                        star_vec.pop();
                        dist_vec.pop();
                        finished = true;
                    }
                    i += 1;
                }
            }
        }
        // Insert current star at the begining of the arrays
        star_vec.insert(0, id_a);
        star_vec.pop();
        dist_vec.insert(0, 0.0);
        dist_vec.pop();
        debug!("  Star vec {:?}", star_vec);
        debug!("  Dist vec {:?}", dist_vec);
        // Calculate center of the polygon
        let mut center_ra_rad = 0.0;
        let mut center_dec_rad = 0.0;
        for star_id in star_vec.iter() {
            center_ra_rad += star_list[*star_id].ra_rad;
            center_dec_rad += star_list[*star_id].dec_rad;
        }
        center_ra_rad = center_ra_rad / POLYGON_EDGES as f32;
        center_dec_rad = center_dec_rad / POLYGON_EDGES as f32;
        // Don't store if polygon already exists
        let mut polygon_exists = false;
        'hexist: for h in polygons.iter() {
            if h.center_ra_rad == center_ra_rad && h.center_dec_rad == center_dec_rad {
                debug!("  !! Polygon already exists: {} = {}", id_a, h.star_index);
                polygon_exists = true;
                break 'hexist;
            }
        }
        if !polygon_exists {
            // Calculate the lengths of the polygon connections
            let mut k = 0;
            for i in 0..star_vec.len() - 1 {
                let star_a = &star_list[star_vec[i]];
                debug!("  Exists - i:{} star_a:{}", i, star_vec[i]);
                for n in (i + 1)..star_vec.len() {
                    // Calculate distance between the stars
                    let star_b = &star_list[star_vec[n]];
                    let length = star_distance_rad(star_a, star_b);
                    debug!("  Exists - {} length from {} to {} = {}", k, i, n, length);
                    if length == 0.0 {
                        debug!("  Exists - {} length 0. star_a:{:?} star_b:{:?}", k, star_vec[i], star_vec[n]);
                    }
                    length_vec[k] = length;
                    k += 1;
                }
            }
            // Sort: https://users.rust-lang.org/t/how-to-sort-a-vec-of-floats/2838
            length_vec.sort_by(|a, b| a.partial_cmp(b).unwrap()); 
            // Normalize the length of the connections by the longest length
            let longest_length = length_vec[length_vec.len() - 1];
            for i in 0..length_vec.len() {
                length_vec[i] = length_vec[i] / longest_length;
            }
            length_vec[0] = longest_length;
            debug!("  Length vec: {:?}, longest_length (rad): {}", length_vec, longest_length);
            // Store polygon data
            let polygon = Polygon {
                star_index: id_a,
                star_list: star_vec,
                length_list: length_vec,
                center_ra_rad: center_ra_rad,
                center_dec_rad: center_dec_rad,
            };
            polygons.push(polygon);
        }
    }
    Some(polygons)
}


fn main() {
    env_logger::init();

    let mut star_list: Vec<Star> = Vec::new();
    match read_stars_from_file(0.0, 0.0, 5.0, 10.0) {
        Ok(star_list_read) => {
            star_list = star_list_read;
        }
        Err(err) => println!("Error {:?}", err),
    }
    for star in &star_list {
        println!("Star id:{}\thip:{}\tra:{} \tdec:{}\tmagnitude:{}", star.id, star.hip, star.ra, star.dec, star.magnitude);
    }
    match find_polygons(&star_list) {
        Some(polygons) => {
            for polygon in polygons {
                println!("{}-gon for star {}: {:?} {:?}", POLYGON_EDGES, polygon.star_index, polygon.length_list, polygon.star_list);
            }
        },
        None => println!("None")
    }
    println!("Star list length: {}", star_list.len());
}