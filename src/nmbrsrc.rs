extern crate rand;
extern crate rand_distr;

use nmbrsrc::rand::{distributions::Uniform, Rng};
use nmbrsrc::rand_distr::{Poisson, Normal, Distribution};
use std::fs::File;
use std::io::{BufRead, Write};
use std::io::BufReader;

//const U40_MAX_VALLUE : f64= 1099511627775.0;

//TODO Filter duplicates?

/// length = number of elements in result
/// mean = mean point of distribution
/// deviation = standard deviation
/// result will be ordered
pub fn get_normal_dist(length: usize, mean: f64, deviation: f64) -> Vec<usize> {
    let normal = Normal::new(mean, deviation).unwrap();
    let mut rng = rand::thread_rng();
    let mut vec = Vec::with_capacity(length);
    for _ in 0..length {
        vec.push(normal.sample(&mut rng) as usize);
    }
    vec.sort();
    vec
}

/// length = number of elements in result
/// result will be ordered
pub fn get_poisson_dist(length: usize, lambda: f64) -> Vec<usize> {
    let poi = Poisson::new(lambda).unwrap();
    let mut rng = rand::thread_rng();
    let mut vec = Vec::with_capacity(length);
    for _ in 0..length {
        let x : u64 = poi.sample(&mut rng);
        vec.push(x as usize);
    }
    vec.sort();
    vec
}

/// length = number of elements in result
/// result will be ordered
pub fn get_uniform_dist(length: usize) -> Vec<usize> {
    let mut vec: Vec<usize> = rand::thread_rng().sample_iter(Uniform::from(0..1099511627775)).take(length).collect();
    vec.sort();
    vec
}

///If File exists, values will be loaded & written sorted with new values. In this case.
///Else a new File will be created
pub fn save(values: &Vec<usize>, path: &str) -> std::io::Result<()> {
    let value_string = if let Ok(mut old_values) = load(path) {
        old_values.append(&mut values.clone());
        old_values.sort(); //TODO theoretisch könnte man hier zeit sparen, wenn man ausnutzt, dass beide vektoren sortiert sind
        format!("{:?}", old_values)
    } else {
        format!("{:?}", values)
    };
    let mut output = File::create(path)?;
    output.write_all(value_string[1..value_string.len() - 1].as_bytes())
}

pub fn load(path: &str) -> std::io::Result<Vec<usize>> {
    let input = File::open(path)?;
    let file_reader = BufReader::new(&input);
    for l in file_reader.lines() {
        let line = l?;
        let v = line.split(",").map(|x| x.trim()).filter(|s| !s.is_empty());
        return Ok(v.into_iter().map(|s| s.parse::<usize>().unwrap()).collect());
    }
    return Ok(vec![]);
}