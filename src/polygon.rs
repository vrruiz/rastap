use log::{debug};

pub const POLYGON_EDGES: usize = 4;
pub const TOLERANCE: f64 = 0.01;

/// Star data structure
#[derive(Clone)]
pub struct Star {
    pub id: u64,
    pub db_id: u64,    // Catalogue reference
    pub ra: f64,       // Right Ascension (R.A.)
    pub dec: f64,      // Declination (Dec)
    pub ra_rad: f64,   // R.A. in radians
    pub dec_rad: f64,  // Dec in radians
    pub magnitude: f64 // Magnitude
}

/// Polygon structure
pub struct Polygon {
    pub star_index: usize,
    pub star_list: Vec<usize>,
    pub length_list: Vec<f64>,
    pub center_ra_rad: f64,
    pub center_dec_rad: f64,
}

/// Calculate the number of vertex connections of a polygon
pub fn polygon_connections(polygon: usize) -> usize {
    let mut sides = 0;
    for i in 1..polygon {
        sides = sides + polygon - i;
    }
    return sides;
}

/// Calculate star distance between two stars
pub fn star_distance_rad(star_a: &Star, star_b: &Star) -> f64 {
    ((star_b.ra_rad - star_a.ra_rad).abs()).sqrt() + ((star_b.dec_rad - star_a.dec_rad).abs()).sqrt()
}

/// Find polygons. For each star, the POLYGON_EDGES-1 closest stars.
pub fn find_polygons(star_list: &Vec<Star>) -> Option<Vec<Polygon>> {
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
        let mut length_vec = vec![0_f64; conn_number];
        let mut dist_vec = vec![f64::MAX; POLYGON_EDGES];
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
        center_ra_rad = center_ra_rad / POLYGON_EDGES as f64;
        center_dec_rad = center_dec_rad / POLYGON_EDGES as f64;
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
                    length_vec[k] = length as f64;
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
            // length_vec[0] = longest_length;
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

/// Compare star database and image polygons
pub fn find_fit(image_polygons: &Vec<Polygon>, star_polygons: &Vec<Polygon>) {
    debug!("Find fit > Searching similar polygons");
    let mut n = 0; // Number of similar polygons found
    for image_pol in image_polygons.iter() {
        for star_pol in star_polygons.iter() {
            let mut diff_list = Vec::new();
            diff_list.resize(star_pol.length_list.len(), 0.0);
            let mut similar = true;
            'length: for i in 0..star_pol.length_list.len() - 1 {
                // Compare the edge lengths. Discard if tolerance is exceeded.
                let a = image_pol.length_list[i];
                let b = star_pol.length_list[i];
                let difference;
                if a > b {
                    difference = b / a;
                } else {
                    difference = a / b;
                }
                diff_list[i] = difference;
                if difference < 0.99 {
                    // debug!("difference: {} a:{} b:{} false", difference, a, b);
                    similar = false;
                    break 'length;
                } else {
                    // debug!("difference: {} a:{} b:{} true", difference, a, b);
                }
            }
            if similar == true {
                println!("Find fit > Similar polygon found\n  image_pol:{:?}\n   star_pol:{:?}\n difference:{:?}", image_pol.length_list, star_pol.length_list, diff_list);
                n += 1;
            }
        }
    }
    debug!("Found {} similar polygons", n);
}