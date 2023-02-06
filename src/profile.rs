#![allow(dead_code, unused_variables)]

//use std::thread::sleep;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct Timers(HashMap<String, Timer>);

#[derive(Debug, Clone)]
pub struct Timer {
    start: Instant,
    last: Duration,
    total: Duration,
    count: u64,
}

impl Default for Timer {
    fn default() -> Self {
        Timer {
            start: Instant::now(),
            last: Duration::default(),
            total: Duration::default(),
            count: 0,
        }
    }
}

impl Timers {
    pub fn new() -> Timers {
        Timers(HashMap::new())
    }

    pub fn start(&mut self, name: &str) {
        #[cfg(debug_assertions)]
        self.0
            .entry(name.to_string())
            .and_modify(|e| e.start = Instant::now())
            .or_insert(Timer::default());
    }

    pub fn stop(&mut self, name: &str) {
        #[cfg(debug_assertions)]
        self.0.entry(name.to_string()).and_modify(|e| {
            e.last = e.start.elapsed();
            e.total += e.last;
            e.count += 1
        });
    }

    pub fn get(&self, name: &str) -> Option<&Timer> {
        #[cfg(debug_assertions)]
        {
            self.0.get(&name.to_string())
        }
        #[cfg(not(debug_assertions))]
        None
    }

    fn do_print(prefix: &str, suffix: &str, name: &str, time: u128) {
        #[cfg(debug_assertions)]
        {
            let m = match prefix {
                "" => String::from(""),
                s => format!("[{s}] "),
            };
            println!("{m}{name}{suffix}: {time}ms");
        }
    }

    pub fn print(&self, name: &str, msg: &str) {
        #[cfg(debug_assertions)]
        {
            if let Some(e) = self.0.get(&name.to_string()) {
                if e.count < 1 { return }
                let t = e.last.as_millis();
                Timers::do_print(msg, "", name, t);
            }
        }
    }

    pub fn print_avg(&self, name: &str, msg: &str) {
        #[cfg(debug_assertions)]
        {
            if let Some(e) = self.0.get(&name.to_string()) {
                if e.count < 1 { return }
                let t = e.total.as_millis() / e.count as u128;
                Timers::do_print(msg, "(avg)", name, t);
            }
        }
    }

    pub fn print_all(&self, msg: &str) {
        #[cfg(debug_assertions)]
        {
            for (n, e) in &self.0 {
                if e.count < 1 { continue }
                let t = e.last.as_millis();
                Timers::do_print(msg, "", n, t);
            }
        }
    }

    pub fn print_all_avg(&self, msg: &str) {
        #[cfg(debug_assertions)]
        {
            for (n, e) in &self.0 {
                if e.count < 1 { continue }
                let t = e.total.as_millis() / e.count as u128;
                Timers::do_print(msg, "(avg)", n, t);
            }
        }
    }
}

/*
fn main() {
    let mut timers = Timers::new();

    timers.start("test");
    #[cfg(debug_assertions)]
    sleep(Duration::from_secs(2));
    timers.stop("test");
    timers.print("test");

    timers.start("test");
    #[cfg(debug_assertions)]
    sleep(Duration::from_secs(2));
    timers.stop("test");
    timers.print("test");

    timers.start("test1");
    #[cfg(debug_assertions)]
    sleep(Duration::from_secs(2));
    timers.stop("test1");

    timers.print_avg("test");
    timers.print_avg("test2");
    #[cfg(debug_assertions)]
    println!("ALL:");
    timers.print_all();

    #[cfg(debug_assertions)]
    {
        println!("{:?}", timers.get("test"));
        println!("{:?}", timers.get("test2"));
    }
}
*/
