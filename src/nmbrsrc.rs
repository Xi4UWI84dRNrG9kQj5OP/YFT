extern crate rand;
extern crate rand_distr;
extern crate serde;
extern crate rmp_serde as rmps;

use self::rand::{distributions::Uniform, Rng};
use self::rand_distr::{Poisson, Normal, Distribution};
use std::fs::File;
use self::serde::{Serialize, Deserialize};
use self::rmps::{Serializer, Deserializer};

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
        let x: u64 = poi.sample(&mut rng);
        vec.push(x as usize);
    }
    vec.sort();
    vec
}

/// length = number of elements in result
/// n = distribution power
/// result will be ordered
pub fn get_power_law_dist(length: usize, n: f64) -> Vec<usize> {
    //from http://mathworld.wolfram.com/RandomNumber.html
    //have to be float cause else power gets to big
    let x0: f64 = 1.;
    let x1: f64 = 1099511627775.;
    let mut rng = rand::thread_rng();
    let mut vec = Vec::with_capacity(length);
    let subterm_0 = x0.powf(n + 1.);
    let subterm_1 = x1.powf(n + 1.) - subterm_0;
    let subterm_2 = 1. / (n + 1.);
    for _ in 0..length {
        let y : f64 = rng.gen();
        let subterm_3 : f64 = subterm_1 * y + subterm_0;
        vec.push(subterm_3.powf(subterm_2) as usize);
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
    if let Ok(mut old_values) = load(path) {
        old_values.append(&mut values.clone());
        old_values.sort(); //TODO theoretisch könnte man hier zeit sparen, wenn man ausnutzt, dass beide vektoren sortiert sind
        let mut output = File::create(path)?;
        Ok(old_values.serialize(&mut Serializer::new(&mut output)).unwrap())
    } else {
        let mut output = File::create(path)?;
        Ok(values.serialize(&mut Serializer::new(&mut output)).unwrap())
    }
}

pub fn load(path: &str) -> std::io::Result<Vec<usize>> {
    let input = File::open(path)?;
    let mut deserializer = Deserializer::new(input);
    let values: Vec<usize> = Deserialize::deserialize(&mut deserializer).unwrap();
    Ok(values)
}