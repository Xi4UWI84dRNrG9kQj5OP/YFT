extern crate stats_alloc;

use std::alloc::System;
//Global
use self::stats_alloc::{StatsAlloc, INSTRUMENTED_SYSTEM, Region};
use std::time::Instant;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

pub struct Log<'a> {
    run_name: String,
    memlog: Option<Memlog<'a>>,
    timelog: Option<Timelog>,
}

struct Timelog {
    start: Instant,
    last: Instant,
}

struct Memlog<'a> {
    reg: Region<'a, System>,
}

impl<'a> Log<'a> {
    pub fn new(run_name: String) -> Log<'a> {
        Log { run_name: run_name, memlog: None, timelog: None }
    }

    pub fn set_log_time(&mut self) -> &mut Log<'a> {
        self.timelog = Some(Timelog { start: Instant::now(), last: Instant::now() });
        self
    }

    pub fn set_log_mem(&mut self) -> &mut Log<'a> {
        self.memlog = Some(Memlog { reg: Region::new(&GLOBAL) });
        self
    }

    pub fn log_time(&mut self, info: &str) -> &mut Log<'a> {
        if let Some(time) = self.timelog.as_ref() {
            self.print_result(format!("info={}\ttime_since_last_call={}\ttime_since_start={}", info, time.start.elapsed().as_millis(), time.last.elapsed().as_millis()));
        } else { //rusts borrow checker can be ugly
            return self;
        }
        if let Some(time) = self.timelog.as_mut() {
            time.last = Instant::now();
        }
        self
    }

    pub fn log_mem(&mut self, info: &str) -> &mut Log<'a> {
        if let Some(mem) = self.memlog.as_ref() {
            let stats = mem.reg.change();
            self.print_result(format!("info={}\tbytes_allocated={}\tmax_bytes_allocated={}", info, stats.bytes_allocated - stats.bytes_deallocated as usize, stats.bytes_max_used as usize));
        }
        self
    }

    //values must not be empty and have to be in format "value_name=value\tvalue2_name=value2[..]"
    pub fn print_result(&self, values: String) {
            println!("RESULT\trun={}\t{}", self.run_name, values);
    }
}