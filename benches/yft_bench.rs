#[macro_use]
extern crate criterion;
extern crate yft;
extern crate uint;

//external use
use criterion::Criterion;
//use criterion::black_box;
use criterion::BenchmarkId;
use criterion::Throughput;
use std::time::Duration;
use uint::u40;

pub fn criterion_benchmark(c: &mut Criterion) { //TODO das muss auch mit parametern gehen!
    let values = yft::nmbrsrc::get_uniform_dist(10000000);
//    let yft = yft::yft40::YFT::new(values.into_iter().map(|v| u40::from(v)).collect(), &mut None, 8, 10);
    let yft = yft::yft::YFT::new(values, &mut None, 8, 10);

    let mut group = c.benchmark_group("query");
    for s in yft::nmbrsrc::get_uniform_dist(5).iter() {
        group.throughput(Throughput::Bytes(*s as u64));
//        group.sample_size(10);
        group.measurement_time(Duration::from_millis(100));
        group.warm_up_time(Duration::from_millis(100));
        group.bench_with_input(BenchmarkId::from_parameter(s), s, |b, &s| {
//            b.iter(|| yft.predecessor(u40::from(s)));
            b.iter(|| yft.predecessor(s));
        });
//        group.bench_with_input(BenchmarkId::from_parameter(s), s, |b, &s| {
//            b.iter(|| pred(&values,s));
//        });
    }
    group.finish();
}

///binary search predecessor
fn pred(element_list: &Vec<usize>, element: usize) -> Option<usize> {
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

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);