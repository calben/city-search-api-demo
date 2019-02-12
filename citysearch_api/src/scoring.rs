extern crate geo;
extern crate strsim;

use geo::prelude::*;
use geo::Point;
use strsim::jaro_winkler;

const EARTH_CIRCUMFERENCE: i64 = 40_075_000;
const MAX_POPULATION: i64 = 8_175_133;

pub fn position_score(a: Point<f64>, b: Point<f64>) -> f64 {
    // 40 million is for earth circumfrence
    // plus a little for computation error
    // this is the expected maximum city distance
    // guaranteeing this value is always < 1
    1.0 - (a.vincenty_distance(&b).unwrap() / EARTH_CIRCUMFERENCE as f64)
}

//
// http://users.cecs.anu.edu.au/~Peter.Christen/publications/tr-cs-06-02.pdf
// WINKLER PERFORMS WELL ACCORDING TO THIS PAPER
//
pub fn name_score(q: &str, names: &Vec<String>) -> f64 {
    names
        .iter()
        .map(|a| jaro_winkler(q, a))
        .min_by(|a, b| b.partial_cmp(a).unwrap())
        .unwrap()
}

pub fn population_score(population: i64) -> f64 {
    // city population divided by the dervied maximum
    population as f64 / MAX_POPULATION as f64
}
