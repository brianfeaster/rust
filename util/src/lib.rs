//! # Local Library
//!
//! Often used one-off functions

use ::std::{
    io::{Write},
    time,
    io,
    thread,
    fmt
};

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
    thread::sleep(time::Duration::from_millis(s));
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