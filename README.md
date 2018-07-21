## SRTM for Rust

Reads elevation data from ``.hgt`` files in Rust. Supports resolutions of 1 angle second (SRTM1) and 3 angle-seconds (SRTM3).

## Example

    extern crate srtm;

    let tile = srtm::Tile::from_file("N35E138.hgt").unwrap();
