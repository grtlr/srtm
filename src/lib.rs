extern crate byteorder;

use std::fs;
use std::fs::File;

use std::io;
use std::io::{Read, BufReader};
use std::path::Path;
use byteorder::{BigEndian, ReadBytesExt};

#[derive(PartialEq,Eq,Clone,Copy,Debug)]
pub enum Resolution {
    SRTM1,
    SRTM3,
}

#[derive(Debug)]
pub struct Tile {
    pub latitude: i32,
    pub longitude: i32,
    pub resolution: Resolution,
    data: Vec<i16>,
}

#[derive(Debug)]
pub enum Error {
    ParseLatLong,
    Filesize,
    Read,
}

impl Tile {
    fn new_empty(lat: i32, lng: i32, res: Resolution) -> Tile {
        Tile {
            latitude: lat,
            longitude: lng,
            resolution: res,
            data: Vec::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Tile, Error> {
        let (lat, lng) = get_lat_long(&path)?;
        let res = get_resolution(&path).ok_or(Error::Filesize)?;
        let file = File::open(&path).map_err(|_| Error::Read)?;
        let reader = BufReader::new(file);
        let mut tile = Tile::new_empty(lat, lng, res);
        tile.data = parse(reader, tile.resolution).map_err(|_| Error::Read)?;
        Ok(tile)
    }

    pub fn extent(&self) -> u32 {
        match self.resolution {
            Resolution::SRTM1 => 3601,
            Resolution::SRTM3 => 1201,
        }
    }

    pub fn max_height(&self) -> i16 {
        *(self.data.iter().max().unwrap())
    }

    pub fn get(&self, x: u32, y: u32) -> i16 {
        self.data[self.idx(x, y)]
    }

    fn idx(&self, x: u32, y: u32) -> usize {
        assert!(x < self.extent() && y < self.extent());
        (y * self.extent() + x) as usize
    }
}

fn get_resolution<P: AsRef<Path>>(path: P) -> Option<Resolution> {
    let from_metadata = |m: fs::Metadata| match m.len() {
        25934402 => Some(Resolution::SRTM1),
        2884802 => Some(Resolution::SRTM3),
        _ => None,
    };
    fs::metadata(path).ok().and_then(from_metadata)
}

// FIXME Better error handling.
fn get_lat_long<P: AsRef<Path>>(path: P) -> Result<(i32, i32), Error> {
    let stem = path.as_ref().file_stem().ok_or(Error::ParseLatLong)?;
    let desc = stem.to_str().ok_or(Error::ParseLatLong)?;
    if desc.len() != 7 {
        return Err(Error::ParseLatLong);
    }

    let get_char = |n| desc.chars().nth(n).ok_or(Error::ParseLatLong);
    let lat_sign = if get_char(0)? == 'N' { 1 } else { -1 };
    let lat: i32 = desc[1..3].parse().map_err(|_| Error::ParseLatLong)?;

    let lng_sign = if get_char(3)? == 'E' { 1 } else { -1 };
    let lng: i32 = desc[4..7].parse().map_err(|_| Error::ParseLatLong)?;
    Ok((lat_sign * lat, lng_sign * lng))
}

fn total_size(res: Resolution) -> u32 {
    match res {
        Resolution::SRTM1 => 3601 * 3601,
        Resolution::SRTM3 => 1201 * 1201,
    }
}

fn parse<R: Read>(reader: R, res: Resolution) -> io::Result<Vec<i16>> {
    let mut reader = reader;
    let mut data = Vec::new();
    for _ in 0..total_size(res) {
        let h = try!(reader.read_i16::<BigEndian>()) as i16;
        data.push(h);
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::get_lat_long;

    #[test]
    fn parse_latitute_and_longitude() {
        let ne = Path::new("N35E138.hgt");
        assert_eq!(get_lat_long(&ne).unwrap(), (35, 138));

        let nw = Path::new("N35W138.hgt");
        assert_eq!(get_lat_long(&nw).unwrap(), (35, -138));

        let se = Path::new("S35E138.hgt");
        assert_eq!(get_lat_long(&se).unwrap(), (-35, 138));

        let sw = Path::new("S35W138.hgt");
        assert_eq!(get_lat_long(&sw).unwrap(), (-35, -138));
    }
}
