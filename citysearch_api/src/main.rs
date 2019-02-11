// web framework
extern crate actix;
extern crate actix_web;
extern crate env_logger;

// json and db
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate postgres;
#[macro_use]
extern crate lazy_static;
extern crate geo;
extern crate strsim;

use std::cmp::min;
use std::str::FromStr;
use std::thread;

use argparse::{ArgumentParser, Store, StoreTrue};
use strsim::jaro_winkler;

use actix_web::http::Method;
use actix_web::{middleware, server, App, HttpResponse, Query};
use geo::prelude::*;
use geo::Point;
use postgres::{Connection, TlsMode};
use std::collections::HashMap;
// use chrono::{NaiveDate, NaiveDateTime};
// use postgis::ewkb::EwkbPoint;

#[derive(Copy)]
enum DataSource {
    Postgres,
    Memory,
}

impl Clone for DataSource {
    fn clone(&self) -> DataSource {
        *self
    }
}

impl FromStr for DataSource {
    type Err = ();

    fn from_str(s: &str) -> Result<DataSource, ()> {
        match s {
            "postgres" => Ok(DataSource::Postgres),
            "memory" => Ok(DataSource::Memory),
            _ => Err(()),
        }
    }
}

lazy_static! {
    static ref CITYRECORDS: HashMap<u32, Vec<CityRecord>> = {
        let mut m = HashMap::new();
        m.insert(0, get_city_records_from_db());
        m
    };
}

// should have used this, but didn't want to spend too much more time on this!
#[derive(Clone, Debug)]
struct CityScore {
    query: SuggestionParam,
    name_score: f64,
    population_score: f64,
    position_score: f64,
}

#[derive(Debug, Clone)]
struct CityRecord {
    id: i32,
    name: String,
    alt_names: Vec<String>,
    lat: f64,
    long: f64,
    position: Point<f64>,
    population: i64,
}

impl CityRecord {
    fn to_cityresult(&self, search_term: &str, position: Option<Point<f64>>) -> CityResult {
        CityResult {
            name: self.name.clone(),
            lat: self.lat,
            long: self.long,
            score: self.score(search_term, position),
        }
    }

    fn score(&self, search_term: &str, position: Option<Point<f64>>) -> f64 {
        // let mut a: &str;
        // let mut b: &str;
        // if self.name.len() >= search_term.len() {
        //     a = &self.name[..search_term.len()].as_ref();
        //     b = search_term;
        // } else {
        //     a = &self.name.as_ref();
        //     b = (String::from(search_term)[..(&self.name.len())]).as_ref();
        // }
        let longest = min(self.name.len(), search_term.len());
        let mut v = Vec::new();
        v.push(
            self.name
                .to_lowercase()
                .chars()
                .take(longest)
                .collect::<String>(),
        );
        for name in &self.alt_names {
            let shortened = name
                .to_lowercase()
                .chars()
                .take(longest)
                .collect::<String>();
            v.push(shortened);
        }
        println!("{:?}", v);
        let b = &search_term.to_string().to_lowercase()[..longest];
        //
        // http://users.cecs.anu.edu.au/~Peter.Christen/publications/tr-cs-06-02.pdf
        // WINKLER PERFORMS WELL ACCORDING TO THIS PAPER
        //
        let name_distance_score = v
            .iter()
            .map(|a| jaro_winkler(a, b))
            .min_by(|a, b| b.partial_cmp(a).unwrap())
            .unwrap();
        match position {
            // note that this still uses the name distance as the "priority" difference
            // the population and distance scores will just work as tiebreakers
            // it is possible to make these more equal
            // but it would mean making 3 passes on all the values
            // and probably wouldn't improve our results much
            Some(p) => {
                if 0.6 * name_distance_score
                    + 0.3 * self.positional_distance_score(p)
                    + 0.1 * self.population_score()
                    > 0.55
                {
                    println!(
                        "name distance score between {:?} and {} is {}, position is {:?} to {:?} at {}, population score is {}",
                        v,
                        b,
                        name_distance_score,
                        self.position,
                        p,
                        self.positional_distance_score(p),
                        self.population_score()
                    );
                }
                0.6 * name_distance_score
                    + 0.3 * self.positional_distance_score(p)
                    + 0.1 * self.population_score()
            }
            None => {
                if 0.8 * name_distance_score + 0.2 * self.population_score() > 0.55 {
                    println!(
                        "name distance score between {:?} and {} is {} and the population score is {}",
                        v,
                        b,
                        name_distance_score,
                        self.population_score()
                    );
                }
                0.8 * name_distance_score + 0.2 * self.population_score()
            }
        }
    }

    fn positional_distance_score(&self, b: Point<f64>) -> f64 {
        // 40 million is for earth circumfrence
        // plus a little for computation error
        // this is the expected maximum city distance
        // guaranteeing this value is always < 1
        1.0 - (self.position.vincenty_distance(&b).unwrap() / 40_075_000.0)
    }

    // theoretically can be precomputed
    fn population_score(&self) -> f64 {
        // city population divided by the dervied maximum
        (self.population as f64) / 4612191.0
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SuggestionsResult {
    suggestions: Vec<CityResult>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CityResult {
    name: String,
    lat: f64,
    long: f64,
    score: f64,
}

impl Default for CityResult {
    fn default() -> CityResult {
        CityResult {
            name: String::from("Name"),
            lat: 0.0,
            long: 0.0,
            score: 0.0,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
struct SuggestionParam {
    q: String,
    lat: Option<f64>,
    long: Option<f64>,
}

fn get_city_records_from_db() -> Vec<CityRecord> {
    let conn = Connection::connect(
        "postgres://postgres:0xd04199ee@localhost:5432/citysearch",
        TlsMode::None,
    )
    .unwrap();
    let stmt = conn
        .prepare("select id, name, alt_name, lat, long, population from citysearch.city;")
        .unwrap();
    stmt.query(&[])
        .unwrap()
        .iter()
        .map(|row| CityRecord {
            id: row.get(0),
            name: row.get(1),
            alt_names: row.get(2),
            lat: row.get(3),
            long: row.get(4),
            position: Point::<f64>::from((row.get(4), row.get(3))),
            population: row.get(5),
        })
        .collect::<Vec<_>>()
}

fn get_suggestions_postgres(query: Query<SuggestionParam>) -> HttpResponse {
    println!("{:?}", query);
    let conn = Connection::connect(
        "postgres://postgres:0xd04199ee@localhost:5432/citysearch",
        TlsMode::None,
    )
    .unwrap();
    let stmt = conn.prepare(format!("select (city).name, (city).long, (city).lat from citysearch.all_city_name_distances({}) order by name_distance limit 10;", query.q).as_ref()).unwrap();
    let _result = serde_json::to_string_pretty(&SuggestionsResult {
        suggestions: stmt
            .query(&[])
            .unwrap()
            .iter()
            .map(|row| CityResult {
                name: row.get(0),
                long: row.get(1),
                lat: row.get(2),
                ..Default::default()
            })
            .collect::<Vec<_>>(),
        // thank you rust for iter, map, collect <3
    });
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(_result.unwrap())
}

fn get_suggestions_memory(query: Query<SuggestionParam>) -> HttpResponse {
    println!("Received request {:?}", query);
    let q = query.q.clone();
    let position = if query.lat.is_some() && query.long.is_some() {
        Some(Point::<f64>::from((
            query.long.unwrap(),
            query.lat.unwrap(),
        )))
    } else {
        None
    };

    // PARALLEL
    // TODO NEED TO SOLVE ONE DEFERENCE BUG
    // let chunked_data = CITYRECORDS.get(&0).unwrap().chunks(250);
    // let mut children = vec![];
    // for (_i, city_records) in chunked_data.enumerate() {
    //     // overshadowing q for each thread
    //     let q = query.q.clone();
    //     children.push(thread::spawn(move || -> Vec<CityResult> {
    //         // have to create a copy of this
    //         let mut thread_records: Vec<CityResult> = city_records
    //             .to_vec()
    //             .iter()
    //             .map(move |record| record.to_cityresult(&q.as_ref(), position))
    //             .collect();
    //         thread_records.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    //         let result = thread_records[..15].to_vec().clone();
    //         // println!("processed segment {}, result={:?}", i, result);
    //         result
    //     }));
    // }
    // let mut joined_results = Vec::new();
    // for child in children {
    //     let intermediate_vec = child.join().unwrap();
    //     joined_results.extend(intermediate_vec.iter());
    // }
    // joined_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    // let top_results = joined_results[..15].to_vec();

    // SINGLE THREADED
    let mut all_results: Vec<CityResult> = CITYRECORDS
        .get(&0)
        .unwrap()
        .iter()
        .map(move |record| record.to_cityresult(&q.as_ref(), position))
        .collect();
    all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    let top_results = all_results[..15].to_vec();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            serde_json::to_string_pretty(&SuggestionsResult {
                suggestions: top_results,
            })
            .unwrap(),
        )
}

fn index(query: Query<SuggestionParam>) -> HttpResponse {
    println!("model: {:?}", query);
    HttpResponse::Ok().body("Received") // <- send response
}

fn main() {
    // using argparse
    // it isn't as full featured as another project called clap
    // but argparse seemed to be able to do the job quickly enough
    let mut verbose = false;
    let mut data_source = DataSource::Postgres;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("City Search Rest API Server Main Process");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Log events to console");
        ap.refer(&mut data_source).add_option(
            &["--data-source"],
            Store,
            "Indicate source of data as one of postgres or memory",
        );
        ap.parse_args_or_exit();
    }

    // not sure this is the best way to do this
    let get_suggestions = match data_source {
        DataSource::Postgres => get_suggestions_postgres,
        DataSource::Memory => {
            // initializing our lazy static value for city records to store them in memory
            get_suggestions_memory
        }
    };

    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("city-search-api");

    server::new(move || {
        // below ifelse is messy or requires mutable variable.  asked if there is a cleaner alternative
        // https://www.reddit.com/r/rust/comments/ap4bd0/including_lines_of_code_or_calls_in_method_chain/
        // last checked 2019-02-10 09:27:35
        // checked after lunch, no solution 2019-02-10 15:02:24
        if verbose {
            App::new()
                .middleware(middleware::Logger::default())
                .resource("/suggestions", move |r| {
                    r.method(Method::GET).with(get_suggestions)
                })
        } else {
            App::new().resource("/suggestions", move |r| {
                r.method(Method::GET).with(get_suggestions)
            })
        }
        .resource("/", |r| r.method(Method::GET).with(index))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
