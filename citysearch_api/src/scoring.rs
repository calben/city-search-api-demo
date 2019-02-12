extern crate geo;
extern crate strsim;

use geo::prelude::*;
use geo::Point;
use strsim::jaro_winkler;

const HALF_EARTH_CIRCUMFERENCE: i64 = 20_037_500;
const MAX_POPULATION: i64 = 8_175_133;

pub fn position_score(a: Point<f64>, b: Point<f64>) -> f64 {
    // 40 million is for earth circumfrence
    // plus a little for computation error
    // this is the expected maximum city distance
    // guaranteeing this value is always < 1
    1.0 - (a.vincenty_distance(&b).unwrap() / (HALF_EARTH_CIRCUMFERENCE as f64))
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

// TEST DATA SAMPLE
//        name        |   lat    |    long    | population
// -------------------+----------+------------+------------
//  New York City     | 40.71427 |  -74.00597 |    8175133
//  Toronto           | 43.70011 |   -79.4163 |    4612191
//  Los Angeles       | 34.05223 | -118.24368 |    3792621
//  MontrΘal          | 45.50884 |  -73.58781 |    3268513
//  Chicago           | 41.85003 |  -87.65005 |    2695598
//  Brooklyn          |  40.6501 |  -73.94958 |    2300664
//  Borough of Queens | 40.68149 |  -73.83652 |    2272771
//  Houston           | 29.76328 |  -95.36327 |    2099451
//  Vancouver         | 49.24966 | -123.11934 |    1837969
//  Philadelphia      | 39.95234 |  -75.16379 |    1526006

#[cfg(test)]
mod tests {
    extern crate approx;

    use super::*;

    #[test]
    fn test_vincenty_distance() {
        let a = Point::<f64>::new(17.072561, 48.154563);
        let b = Point::<f64>::new(17.072562, 48.154564);
        assert_relative_eq!(
            a.vincenty_distance(&b).unwrap(),
            0.13378944117648012,
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn test_new_york_to_toronto() {
        let nyc = Point::<f64>::from((-74.00597, 40.71427));
        let to = Point::<f64>::from((-79.4163, 43.70011));
        assert_relative_eq!(
            nyc.vincenty_distance(&to).unwrap(),
            550_000.0,
            epsilon = 1.0e4
        );
    }

    #[test]
    fn test_new_york_to_toronto_position_score() {
        let nyc = Point::<f64>::from((-74.00597, 40.71427));
        let to = Point::<f64>::from((-79.4163, 43.70011));
        assert_relative_eq!(position_score(nyc, to), 0.972, epsilon = 1.0e-2);
    }

    #[test]
    fn test_new_york_to_houston_position_score() {
        let nyc = Point::<f64>::from((-74.00597, 40.71427));
        let houston = Point::<f64>::from((-95.36327, 29.76328));
        assert_relative_eq!(position_score(nyc, houston), 0.886, epsilon = 1.0e-2);
    }

    #[test]
    fn test_name_score_high() {
        let names = vec!["aaa".to_string(), "naw".to_string(), "raw".to_string()];
        let q = "row";
        assert_relative_eq!(name_score(q, &names), 0.8, epsilon = 1.0e-2);
    }

    #[test]
    fn test_name_score_low() {
        let names = vec!["new".to_string(), "naw".to_string(), "raw".to_string()];
        let q = "aaa";
        assert_relative_eq!(name_score(q, &names), 0.55, epsilon = 1.0e-2);
    }

    #[test]
    fn test_name_score_0() {
        let names = vec!["new".to_string(), "naw".to_string(), "raw".to_string()];
        let q = "zzz";
        assert_relative_eq!(name_score(q, &names), 0.0, epsilon = 1.0e-2);
    }

    #[test]
    fn test_name_score_utf8() {
        let names = vec!["new".to_string(), "naw".to_string(), "raw".to_string()];
        let q = "γνω";
        assert_relative_eq!(name_score(q, &names), 0.0, epsilon = 1.0e-2);
    }

}
