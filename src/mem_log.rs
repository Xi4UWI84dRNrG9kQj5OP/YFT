extern crate stats_alloc;
extern crate gnuplot;

use std::alloc::System;
use std::fs::File;
use std::io::{BufRead, Write, BufReader};
//Global
use self::stats_alloc::{StatsAlloc, INSTRUMENTED_SYSTEM, Region};
use self::gnuplot::{Figure, AxesCommon};
use self::gnuplot::Coordinate::Graph;
use self::gnuplot::PlotOption::Caption;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

pub struct Memlog<'a> {
    reg: Region<'a, System>,
    output: Option<File>,
}

impl<'a> Memlog<'a> {
    /// file to log memory usage to (else log to outline)
    /// short log only usage without allocation details
    pub fn new(file: Option<String>) -> Memlog<'a> {
        let output =
            if let Some(file) = file {
                Some(File::create(file).unwrap())
            } else {
                None
            };
        Memlog { reg: Region::new(&GLOBAL), output }
    }

    /// logs the actual memory usage with the given info to the outline.
    pub fn print(&self, info: &str) {
        let stats = self.reg.change();
        println!("Memory usage at {}: {:#?}mb, max usage was: {}mb", info, (stats.bytes_allocated - stats.bytes_deallocated as usize) / 1000000, stats.bytes_max_used as usize / 1000000);
    }

    /// logs the actual memory usage with the given info. If no File is set it will be printed to the outline.
    pub fn log(&mut self, info: &str) {
        if let Some(mut output) = &self.output.as_ref() {
            let stats = self.reg.change();
            let bytes = stats.bytes_allocated + stats.bytes_reallocated as usize - stats.bytes_deallocated as usize;
            let result = output.write_all(format!("{:#?};MB;{}\n", bytes / 1000000, info).as_bytes());
            if let Err(e) = result {
                println!("Logging Error {:#?}", e);
            }
        } else {
            self.print(info);
        }
    }

    ///file have to be read, cause els memlogs would have to stay in RAM -> would be useless
    pub fn plot(&mut self, input_file: String, plot_title: &str) {
        let input = File::open(input_file).unwrap();
        let file_reader = BufReader::new(&input);
        let mut values: Vec<u64> = vec![];
        for l in file_reader.lines() {
            let line = l.unwrap();
            let v = line.split(";").next().unwrap();
            values.push(v.parse::<u64>().unwrap())
        }

        let x: Vec<usize> = (0..values.len() - 1).collect();

        let mut fg = Figure::new();
        fg.axes2d()
            .set_title(plot_title, &[])
            .set_legend(Graph(0.5), Graph(0.9), &[], &[])
            .set_x_label("x", &[])
            .set_y_label("y", &[])
            .lines(
                &x,
                &values,
                &[Caption(plot_title)],
            );
        fg.show();
    }
}