use std::time::Instant;

pub struct Timelog{
    start: Instant,
    last: Instant
}

impl Timelog{
    pub fn new() -> Timelog{
        Timelog { start : Instant::now(), last : Instant::now()}
    }

    pub fn log(&mut self, info: &str){
        println!("Time since start: {}, since last log: {} {}", self.start.elapsed().as_millis(), self.last.elapsed().as_millis(), info);
        self.last = Instant::now();
    }
}