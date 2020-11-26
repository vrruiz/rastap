use std::{
    error::Error,
    fs::File,
    io::{BufReader, Read},
    result::Result
};
use log::{debug};

use byteorder::ByteOrder;
use byteorder::LittleEndian;

use crate::math;
use crate::polygon;

// Mini Gaia DR2 record struct (28 bytes)
// #[repr(C)]
// struct DbStar {
//     id: u64,
//     ra: f64,
//     dec: f64,
//     magnitude: f32
// }

/// Reads Gaia DR2 star database CSV file to memory
pub fn read_stars_from_file(ra_center: f64, dec_center: f64, radii: f64, magnitude_limit: f64) -> Result<Vec<polygon::Star>, Box<dyn Error>> {
    let ra_center_rad = math::hours_to_radians(ra_center);
    let dec_center_rad = dec_center.to_radians();
    let radii_rad = radii.to_radians();

    // Read database
    let mut star_list: Vec<polygon::Star> = Vec::new();
    let file = File::open("mini-gaia-dr2.db").unwrap();
    let mut reader = BufReader::new(file); // Buffered read
    // Read headers
    let mut headers = Vec::<String>::new();
    for _i in 0..3 {
        let mut length = [0u8;1];
        let mut string = [0u8;255];
        reader.read_exact(&mut length).unwrap();
        reader.read_exact(&mut string).unwrap();
        headers.push(String::from_utf8(string[0..length[0] as usize].to_vec()).unwrap());
    }
    // TODO: Parse headers
    // Read stars
    let mut star_bin = [0u8;28];
    let mut n = 0u64;
    loop {
        match reader.read_exact(&mut star_bin) {
            Ok(_) => (),
            Err(e) => {
                // Let's suppose this is the end of the file
                break;
            }
        }
        // Initialize record
        let mut star = polygon::Star {
            id: n,
            db_id: LittleEndian::read_u64(&star_bin[0..8]),
            ra: LittleEndian::read_f64(&star_bin[8..16]) / 360.0 * 24.0, // Convert from degrees to hours
            dec: LittleEndian::read_f64(&star_bin[16..24]),
            ra_rad: 0.0,
            dec_rad: 0.0,
            magnitude: LittleEndian::read_f32(&star_bin[24..28]) as f64,
        };
        // Transform degrees/hours to radians
        star.ra_rad = math::hours_to_radians(star.ra);
        star.dec_rad = star.dec.to_radians();
        // Calculate angular separation between star and center
        let sep_rad = math::angular_separation_radians(ra_center_rad, dec_center_rad, star.ra_rad, star.dec_rad);
        // Filter by magnitude and angular separation
        if star.magnitude < magnitude_limit && sep_rad <= radii_rad {
            // Add star to the list
            debug!("STAR: ra:{} dec:{} mag:{} sep:{}", star.ra, star.dec, star.magnitude, sep_rad);
            star_list.push(star);
        }
        n += 1;
    }
    // Sort by magnitude
    star_list.sort_by(|a, b| a.magnitude.partial_cmp(&b.magnitude).unwrap());
    Ok(star_list)
}