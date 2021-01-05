//! # Local Library
//!
//! Often used concepts.

use ::std::collections::{HashMap};
use ::std::collections::hash_map::{DefaultHasher};
use ::core::hash::{BuildHasher};
use ::std::io::{Write};
use ::std::{time::{SystemTime, Duration}, io, thread, fmt};
use ::rand::{*, rngs::*};

////////////////////////////////////////////////////////////////////////////////

pub struct DeterministicHasher { }

impl BuildHasher for DeterministicHasher {
    type Hasher = DefaultHasher;
    fn build_hasher(&self) -> DefaultHasher {
        DefaultHasher::new()
    }
}

pub type HashMapDeterministic = HashMap<(i32, i32), i32, DeterministicHasher>;

pub fn HashMapDeterministicNew () -> HashMapDeterministic {
    HashMap::with_hasher(DeterministicHasher{})
}

////////////////////////////////////////////////////////////////////////////////

/// Pseudo Random Number Generator
/// *©2021 Shrewm™*
/// # Example
/// ```
/// use util::Prng;
/// let gen = Prng::new(12349876);
/// let f = gen.f32(100.0);
/// let u = gen.u32(100);
/// let i = gen.i32(100);
/// let s = gen.usize(100);
/// ```
pub struct Prng {
  rrr: StdRng
}

impl Prng {
    pub fn new(seed:u64) -> Self {
        Prng{rrr: <StdRng as SeedableRng>::seed_from_u64(seed)}
    }
    pub fn f32(&mut self, n: f32)     -> f32   { self.rrr.gen::<f32>() * n }
    pub fn u32(&mut self, n: u32)     -> u32   { self.rrr.gen::<u32>() % n }
    pub fn usize(&mut self, n: usize) -> usize { self.rrr.gen::<usize>() % n }
    pub fn i32(&mut self, n: u32)     -> i32   { (self.rrr.gen::<u32>() % n) as i32 }
}

/// Sleep for a number of milliseconds
/// 
/// *©2020 Shrewm™*
/// # Example
/// ```
/// mod lib;
/// lib::sleep_ms(100);
/// ```
///
pub fn sleep(s: u64) {
    thread::sleep(Duration::from_millis(s));
}


/// Flush STDOUT
/// 
/// *©2020 Shrewm™*
/// # Example
/// ```
/// mod lib;
/// print!("X");
/// lib::flush();
/// ```
///
pub fn flush () {
    match io::stdout().flush() {
        Ok(_) => (),
        Err(e) => {
            sleep(500);
            eprintln!("crate::util::flush {}", e);
        }
    }
}

/// Closure which just returns duration since last evaluation.
pub fn delta () -> impl FnMut()->Duration  {
    let mut epoch = SystemTime::now();
    move || -> Duration {
        let then = epoch;
        epoch = SystemTime::now();
        epoch.duration_since(then).unwrap()
    }
}

#[derive (Debug)]
pub struct Watch {
    // Duration - Relative to instantiation
    epoch: SystemTime,
    ticks: u32,
    // Period - Relative to last mark
    time: SystemTime,
    pub tick: u32,
    // Mark - Static period details
    pub fps: f32
}

impl Watch {
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            epoch:now, ticks: 0, 
            time:now, tick: 0,
            fps: 0.0
        }
    }

    /// Duration since watch creation.
    pub fn duration (&self) -> f32 {
        SystemTime::now().duration_since(self.epoch).unwrap().as_secs_f32()
    }

    // Duration time since last mark.
    pub fn period (&self) -> f32 {
        SystemTime::now().duration_since(self.time).unwrap().as_secs_f32()
    }

    // Increase tick count for this period.
    pub fn tick(&mut self) -> &mut Self {
        self.tick += 1;
        self
    }

    /// Update static fps and ticks values, reset period.
    pub fn mark(&mut self, after:f32) -> Option<&mut Self> {
        let now = SystemTime::now();
        let secs = now.duration_since(self.time).unwrap().as_secs_f32();
        if secs < after {
            None
        } else {
            // Set current details
            self.fps = self.tick as f32 / secs;
            // Reset state
            self.ticks += self.tick;
            self.time = now;
            self.tick = 0;
            Some(self)
        }
    }
}

/*
fn time_diff(from: SystemTime, to: SystemTime) -> i64 {
    match to.duration_since(from) {
        Ok(duration) => duration.as_millis() as i64,
        Err(e) => -(e.duration().as_millis() as i64)
    }
}
*/

/*
pub fn log(callername: &str, message: &str) {
    let _now = SystemTime::now();
    let _epoch = UNIX_EPOCH; // now + Duration::new(0,900000000);  // Cause match Err
    let _timestamp = time_diff(_epoch, _now);
    println!(
        "\x1b[1;34m[{:?} {}]\x1b[0m - {}",
        _timestamp, callername, message
    );
}
*/

/// Bresenham's Line Drawing algorithm.  Cardinal steps are computed 
/// given a 2d line segment.  This is an iterator struct.
pub struct Walk {
  start: [f32;2],
  end: [f32;2],
  st: usize,
  inc: usize,
  x: i32,
  y: i32,
  yx: i32,
  e: i32
}


impl Walk {
    pub fn new (start : &[f32],
                end   : &[f32]) -> Self {
        let mut x = end[0] as i32 - start[0] as i32;
        let mut y = end[1] as i32 - start[1] as i32;
        let ax = x.abs();
        let ay = y.abs();
        let st;
        let inc;

        if ay < ax { //  # Walk X and increment Y
          if 0 < x {
            if 0<y { st=0; inc=7; } else { st=0; inc=1; }
          } else {
            if 0<y { st=4; inc=5; } else { st=4; inc=3; }
          }
          y=ay; x=ax;
        } else { // Walk Y and increment X
          if 0 < y {
            if 0<x { st=6; inc=7; } else { st=6; inc=5; }
          } else {
          if 0<x { st=2; inc=1; } else { st=2; inc=3; }
        }
          y=ax; x=ay;
        }

        y=y+y;
        let e=y-x;
        let yx=y-x-x;

        return Walk{
          start: [start[0], start[1]],
          end: [end[0], end[1]],
          st, inc, x, y, yx, e};
    } // Walk::new

}

// The cardinal direction -> cartesian vector translation table.
const WALK_VECTORS : [[i32; 3];8]= [
    [ 1,  0,  b'-' as i32],  // 0
    [ 1, -1,  b'/' as i32],  // 1
    [ 0, -1,  b'|' as i32],  // 2
    [-1, -1, b'\\' as i32], // 3
    [-1,  0,  b'-' as i32], // 4
    [-1,  1,  b'/' as i32], // 5
    [ 0,  1,  b'|' as i32], // 6
    [ 1,  1, b'\\' as i32]];// 7

/// Returns array containing [x, y, ch] increments that will perform the walk.
impl Iterator for Walk {
  type Item = [i32; 3];
  fn next(&mut self) -> Option<Self::Item> {
      if 0 < self.x {
          self.x -= 1;
          Some(WALK_VECTORS[
            if 0 < self.e {
                 self.e += self.yx; self.inc
            } else {
                 self.e += self.y; self.st
            }])
      } else {
          None
      }
  }
}

/*
impl IntoIterator for Walk {
  type Item = [i32; 3];
  type IntoIter = Walk;

  fn into_iter(self) -> Self::IntoIter {
    self
  }
}
*/

impl fmt::Debug for Walk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Walk")
            .field("start", &self.start)
            .field("end",  &self.end)
            .field("x",  &self.x)
            .field("y",  &self.y)
            .field("yx",  &self.yx)
            .field("e",  &self.e)
            .field("st",  &self.st)
            .field("inc",  &self.inc)
            .finish()
    }
}