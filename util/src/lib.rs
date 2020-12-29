//! # Local Library
//!
//! Often used one-off functions

use ::std::io::{Write};
use ::std::{time::{SystemTime, Duration}, io, thread, fmt};
use std::collections::{HashMap};
use ::piston_window::*;

pub fn ri32(m: u32) -> i32 { (::rand::random::<u32>() % m) as i32 }

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

// Plotter /////////////////////////////////////////////////////////////////////

pub type PlotterPoints = HashMap<(i32, i32), i32>;

pub struct Plotter {
    pub pwin: ::piston_window::PistonWindow,
    pub colors: HashMap::<i32, [f32;4]>,
    pub key: Option<char>,
    pub hm: PlotterPoints
}

impl Plotter {
pub fn new () -> Plotter {
    Plotter {
        pwin: {
            let mut pwin: ::piston_window::PistonWindow =
                ::piston_window::WindowSettings::new("ASCIIRhOIDS", [320, 240])
                .exit_on_esc(true).decorated(true).build().unwrap();
            pwin.set_max_fps(1111);
            pwin
        },
        colors: {
            let mut h = HashMap::new();
            h.insert(0, [0.0, 0.0, 0.0, 1.0]);
            h.insert(1, [0.5, 0.0, 0.0, 1.0]);
            h.insert(2, [0.0, 0.5, 0.0, 1.0]);
            h.insert(3, [0.5, 0.3, 0.0, 1.0]);
            h.insert(4, [0.0, 0.0, 0.5, 1.0]);
            h.insert(5, [0.5, 0.0, 0.5, 1.0]);
            h.insert(6, [0.0, 0.5, 0.5, 1.0]);
            h.insert(7, [0.5, 0.5, 0.5, 1.0]);
            h.insert(8, [0.2, 0.2, 0.2, 1.0]);
            h.insert(9, [1.0, 0.0, 0.0, 1.0]);
            h.insert(10,[0.0, 1.0, 0.0, 1.0]);
            h.insert(11,[1.0, 1.0, 0.0, 1.0]);
            h.insert(12,[0.0, 0.0, 1.0, 1.0]);
            h.insert(13,[1.0, 0.0, 1.0, 1.0]);
            h.insert(14,[0.0, 1.0, 1.0, 1.0]);
            h.insert(15,[1.0, 1.0, 1.0, 1.0]);
            h
        },
        key: None,
        hm: HashMap::new()
    }
} }

impl Plotter {

    /// Set an index's color.
    pub fn color (&mut self, i:i32, c:[f32;4]) -> &mut Self {
        self.colors.insert(i, c);
        self
    }

    /// Plot a point with it's color.
    pub fn insert (&mut self, x:i32, y:i32, c:i32) -> &mut Self {
        self.hm.insert((x,y), c);
        self
    }

    pub fn clear (&mut self) -> &mut Self {
        self.hm.clear();
        self
    }

    /// Render the internal set of points.
    pub fn render (&mut self) -> &mut Self {
        render(self, None);
        self
    }

    /// Render an external set of points.
    pub fn renderhash (&mut self, pts:&PlotterPoints) -> &mut Self {
        render(self, Some(pts));
        self
    }

    /// Compare char with last key pressed
    pub fn iskey (&self, c:char) -> bool {
        match self.key {
            Some(k) => k == c,
            _ => false
        }
    }
}

fn render (
    this: &mut Plotter,
    hmo: Option<&PlotterPoints>
) {
    this.key = None; // Clear last keypressed
    let hm = if let Some(hm) = hmo { hm } else { &this.hm }; // Internal or external hashmap
    if 0 == hm.len() { return } // No pixels, no rendy.
    let mut eventrender :Option<::piston_window::Event> = None;
    while let Some(event) = this.pwin.next() {
        match event {
            Event::Loop(Loop::Render(_args)) => {
                eventrender = Some(event);
                break
            },
            Event::Input( Input::Button( ButtonArgs{state:_, button:Button::Keyboard(k), scancode:_} ), _ ) => {
                this.key = Some(k as u8 as char);
            },
            _ => { }
        }
    }
    if eventrender.is_none() { return }
    let colors = &this.colors;
    this.pwin.draw_2d(
         &eventrender.unwrap(),
        | _c: Context, g: &mut G2d, _d: &mut GfxDevice | {
            let (xmin, xmax, ymin, ymax,  xsize, ysize) = bounding_box(&hm);
            clear(*colors.get(&0).unwrap_or(&[0.0, 0.0, 0.0, 1.0]), g);
            for ((x, y), c) in hm {
                rectangle(
                    *colors.get(c).unwrap_or(&[5.0, 5.0, 5.0, 1.0]),
                    [*x as f64, *y as f64, 1.0, 1.0], // x,y, w,h
                    // The transform matrix to fit all points in window
                    [[2.0/xsize, 0.0, (xmax + xmin + 1.0) / -xsize ],
                        [0.0, 2.0/ysize, (ymax + ymin + 1.0) / -ysize ]],
                    g);
            }
        }
    );
} // fn render

/// Return bounding box for all x,y coordinates in the hashmap of points.
fn bounding_box (hm: &PlotterPoints) -> (f64, f64, f64, f64, f64, f64) {
    let (xmin, xmax, ymin, ymax) =
        hm.iter().fold(
            (std::i32::MAX, std::i32::MIN, std::i32::MAX, std::i32::MIN),
            | mut r, ((x,y),_) | {
                if *x < r.0 { r.0 = *x };
                if r.1 < *x { r.1 = *x };
                if *y < r.2 { r.2 = *y };
                if r.3 < *y { r.3 = *y };
                r
            }
        );
    (xmin as f64, xmax as f64, ymin as f64, ymax as f64, (xmax-xmin) as f64, (ymax-ymin) as f64)
}