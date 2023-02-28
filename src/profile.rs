#![allow(dead_code, unused_variables)]

//use std::thread::sleep;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Default)]
pub struct Timers(HashMap<String, Vec<Timer>>);

#[derive(Debug, Clone)]
pub struct Timer {
    start: Instant,
    time: Duration,
}

impl Default for Timer {
    fn default() -> Self {
        Timer {
            start: Instant::now(),
            time: Duration::default(),
        }
    }
}

impl Timers {
    pub fn new() -> Timers {
        Timers(HashMap::new())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn start(&mut self, name: &str) {
        #[cfg(debug_assertions)]
        self.0
            .entry(name.to_string())
            .and_modify(|e| e.push(Timer::default()))
            .or_insert(vec![Timer::default()]);
    }

    pub fn stop(&mut self, name: &str) {
        #[cfg(debug_assertions)]
        self.0.entry(name.to_string()).and_modify(|e| {
            if let Some(t) = e.last_mut() {
                t.time = t.start.elapsed();
            }
        });
    }

    pub fn get(&self, name: &str) -> Option<&Vec<Timer>> {
        #[cfg(debug_assertions)]
        {
            self.0.get(&name.to_string())
        }
        #[cfg(not(debug_assertions))]
        None
    }

    pub fn get_all(&self) -> &HashMap<String, Vec<Timer>> {
        #[cfg(debug_assertions)]
        {
            &self.0
        }
        #[cfg(not(debug_assertions))]
        None
    }

    fn format(prefix: &str, suffix: &str, name: &str, time: u128, entries: usize) -> String {
        #[cfg(debug_assertions)]
        {
            let m = match prefix {
                "" => String::from(""),
                s => format!("[{s}] "),
            };
            format!("{m}{name}{suffix}: total_time: {time} nsec nr_entries: {entries}")
        }
    }

    pub fn print(&self, name: &str, msg: &str) {
        #[cfg(debug_assertions)]
        {
            if let Some(e) = self.0.get(&name.to_string()) {
                if let Some(t) = e.last() {
                    println!("{}", Timers::format(msg, "", name, t.time.as_nanos(), 1));
                }
            }
        }
    }

    pub fn print_total(&self, name: &str, msg: &str) {
        #[cfg(debug_assertions)]
        {
            if let Some(e) = self.0.get(&name.to_string()) {
                let s: u128 = e.iter().map(|x| x.time.as_nanos()).sum();
                println!(
                    "{}",
                    Timers::format(msg, "", name, s, e.len())
                );
            }
        }
    }

    pub fn print_all(&self, msg: &str) {
        #[cfg(debug_assertions)]
        {
            for (n, e) in &self.0 {
                if let Some(t) = e.last() {
                    println!("{}", Timers::format(msg, "", n, t.time.as_nanos(), 1));
                }
            }
        }
    }

    pub fn print_all_total(&self, msg: &str) {
        #[cfg(debug_assertions)]
        {
            for (n, e) in &self.0 {
                let s: u128 = e.iter().map(|x| x.time.as_nanos()).sum();
                println!("{}", Timers::format(msg, "", n, s, e.len()));
            }
        }
    }

    pub fn print_all_total_to_buf(&self, msg: &str) -> String {
        #[cfg(debug_assertions)]
        {
            let mut buf = Vec::new();
            for (n, e) in &self.0 {
                let s: u128 = e.iter().map(|x| x.time.as_nanos()).sum();
                buf.push(Timers::format(msg, "", n, s, e.len()));
            }
            buf.join("\n")
        }
    }

    pub fn print_all_to_buf(&self, msg: &str) -> String {
        #[cfg(debug_assertions)]
        {
            let mut buf = Vec::new();
            for (n, e) in &self.0 {
                if let Some(t) = e.last() {
                    buf.push(Timers::format(msg, "", n, t.time.as_nanos(), 1));
                }
            }
            buf.join("\n")
        }
    }
}

/*
fn main() {
    let mut timers = Timers::new();

    timers.start("test");
    #[cfg(debug_assertions)]
    sleep(Duration::from_secs(1));
    timers.stop("test");
    timers.print("test", "");

    timers.start("test");
    #[cfg(debug_assertions)]
    sleep(Duration::from_secs(2));
    timers.stop("test");
    timers.print("test", "");

    timers.start("test1");
    #[cfg(debug_assertions)]
    sleep(Duration::from_secs(1));
    timers.stop("test1");

    timers.print_avg("test", "");
    timers.stop("test2");
    timers.print_avg("test2", "");
    #[cfg(debug_assertions)]
    println!("ALL:");
    timers.print_all("");

    #[cfg(debug_assertions)]
    {
        println!("{:?}", timers.get("test"));
        println!("{:?}", timers.get("test2"));
    }

    #[cfg(debug_assertions)]
    println!("{}", timers.print_all_avg_to_buf("vaccel"));
}
*/
