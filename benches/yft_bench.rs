#[macro_use]
extern crate criterion;
extern crate yft;
extern crate uint;
extern crate rand_distr;

//external use
use criterion::Criterion;
//use criterion::black_box;
use criterion::BenchmarkId;
use criterion::Throughput;
use criterion::measurement::WallTime;
use std::time::Duration;
use uint::{u40, UIntPair};
use rand_distr::Uniform;

///get Input to build YFT
fn get_values() -> Vec<usize> {
//    yft::nmbrsrc::get_uniform_dist(10000000)
    yft::nmbrsrc::load("C:\\tmp\\i.dat").unwrap()
}

fn get_queries() -> Vec<usize> {
//    yft::nmbrsrc::get_uniform_dist(5)
    yft::nmbrsrc::load("C:\\tmp\\t.dat").unwrap()
}

fn bench_yft64_long(c: &mut Criterion<WallTime>) {
    for i in 1..10 {
        let elements = get_values().iter().step_by(2).map(|v| v.clone()).collect();
        let yft = yft::yft64::YFT::new(elements, &mut None, 8, 10);
        let mut group = c.benchmark_group("query");
        for s in get_queries().iter() {
            group.throughput(Throughput::Bytes(*s as u64));
//        group.sample_size(10);
            group.measurement_time(Duration::from_millis(1));
            group.warm_up_time(Duration::from_millis(1));
            group.bench_with_input(BenchmarkId::new(format!("bench_yft64_long_{}", i), s), s, |b, &s| {
                b.iter(|| yft.predecessor(s));
            });
        }
        group.finish();
    }
}

fn bench_yft64_short(c: &mut Criterion<WallTime>) {
    let yft = yft::yft64::YFT::new(get_values(), &mut None, 8, 10);
    let mut group = c.benchmark_group("query");
    for s in get_queries().iter() {
        group.throughput(Throughput::Bytes(*s as u64));
//        group.sample_size(10);
        group.measurement_time(Duration::from_millis(1));
        group.warm_up_time(Duration::from_millis(1));
        group.bench_with_input(BenchmarkId::new("bench_yft64_short", s), s, |b, &s| {
            b.iter(|| yft.predecessor(s));
        });
    }
    group.finish();
}

fn bench_yft40_long(c: &mut Criterion<WallTime>) {
    for i in 1..10 {
        let elements = get_values().iter().step_by(2).map(|v| u40::from(v.clone())).collect();
        let yft = yft::yft40::YFT::new(elements, &mut None, 8, 10);
        let mut group = c.benchmark_group("query");
        for s in get_queries().iter() {
            group.throughput(Throughput::Bytes(*s as u64));
//        group.sample_size(10);
            group.measurement_time(Duration::from_millis(1));
            group.warm_up_time(Duration::from_millis(1));
            group.bench_with_input(BenchmarkId::new(format!("bench_yft40_long_{}", i), s), s, |b, &s| {
                b.iter(|| yft.predecessor(u40::from(s)));
            });
        }
        group.finish();
    }
}

fn bench_yft40_short(c: &mut Criterion<WallTime>) {
    let yft = yft::yft40::YFT::new(get_values().into_iter().map(|v| u40::from(v)).collect(), &mut None, 8, 10);
    let mut group = c.benchmark_group("query");
    for s in get_queries().iter() {
        group.throughput(Throughput::Bytes(*s as u64));
//        group.sample_size(10);
        group.measurement_time(Duration::from_millis(1));
        group.warm_up_time(Duration::from_millis(1));
        group.bench_with_input(BenchmarkId::new("bench_yft40_short", s), s, |b, &s| {
            b.iter(|| yft.predecessor(u40::from(s)));
        });
    }
    group.finish();
}

fn bench_bin_search_long(c: &mut Criterion<WallTime>) {
    for i in 1..10 {
        let values = get_values().iter().step_by(2).map(|v| v.clone()).collect();
        let mut group = c.benchmark_group("query");
        group.measurement_time(Duration::from_millis(1));
        group.warm_up_time(Duration::from_millis(1));
        for s in get_queries().iter() {
            group.bench_with_input(BenchmarkId::new(format!("bench_bin_search_long_{}", i), s), s, |b, &s| {
                b.iter(|| bin_search_pred(&values, s));
            });
        }
        group.finish();
    }
}

fn bench_bin_search_short(c: &mut Criterion<WallTime>) {
    let values = get_values();
    let mut group = c.benchmark_group("query");
    group.measurement_time(Duration::from_millis(1));
    group.warm_up_time(Duration::from_millis(1));
    for s in get_queries().iter() {
        group.bench_with_input(BenchmarkId::new("bench_bin_search_short", s), s, |b, &s| {
            b.iter(|| bin_search_pred(&values, s));
        });
    }
    group.finish();
}


///binary search predecessor
fn bin_search_pred(element_list: &Vec<usize>, element: usize) -> Option<usize> {
    let mut l = 0;
    let mut r = element_list.len() - 1;

    while l != r {
        let m = (l + r) / 2;
        if element_list[m] < element {
            r = m
        } else {
            l = m + 1;
        }
    }

    if element >= element_list[l] {
        Some(element_list[l])
    } else {
        None
    }
}

fn bench_u40_ops(c: &mut Criterion<WallTime>) {
    let mut group = c.benchmark_group("query");
    group.measurement_time(Duration::from_millis(1));
    group.warm_up_time(Duration::from_millis(1));
    for s in get_queries().iter() {
        group.bench_with_input(BenchmarkId::new("bench_u40_ops", s), s, |b, &_s| {
            b.iter(|| vec![u40::from(0); 1000]);
        });
    }
    group.finish();
}

fn bench_u64_ops(c: &mut Criterion<WallTime>) {
    let mut group = c.benchmark_group("query");
    group.measurement_time(Duration::from_millis(1));
    group.warm_up_time(Duration::from_millis(1));
    for s in get_queries().iter() {
        group.bench_with_input(BenchmarkId::new("bench_u64_ops", s), s, |b, &_s| {
            b.iter(|| vec![0; 1000]);
        });
    }
    group.finish();
}

criterion_group!(bench_yft64_long_group, bench_yft64_long);
criterion_group!(bench_yft40_long_group, bench_yft40_long);
criterion_group!(bench_bin_search_long_group, bench_bin_search_long);
criterion_group!(bench_yft64_short_group, bench_yft64_short);
criterion_group!(bench_yft40_short_group, bench_yft40_short);
criterion_group!(bench_bin_search_short_group, bench_bin_search_short);
criterion_group!(bench_ops_group, bench_u40_ops, bench_u64_ops);
criterion_main!(bench_yft64_long_group,bench_yft64_short_group, bench_yft40_long_group,bench_yft40_short_group, bench_bin_search_long_group,bench_bin_search_short_group, bench_ops_group);