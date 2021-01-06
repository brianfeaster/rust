//! Piston based point plotter.
//!
//! Create/render points->color hash maps.
//! Doubles as a free HashMap although you can pass your own to render.
//! 
use ::std::collections::{HashMap};
use ::piston_window::*;
use ::util::*;

pub struct Plotter {
    pub pwin: PistonWindow,
    pub colors: HashMap::<i32, [f32;4]>,
    pub key: Option<char>,
    pub hm: HashMapDeterministic
}

impl Plotter {

pub fn new () -> Plotter {
    Plotter {
        pwin: {
            let mut pwin: PistonWindow =
                WindowSettings::new("ASCIIRhOIDS", [640, 480])
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
        hm: HashMap::with_hasher(DeterministicHasher{})
    }
}

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
    _render(self, None);
    self
}

/// Render an external set of points.
pub fn renderhash (&mut self, pts:&HashMapDeterministic) -> &mut Self {
    _render(self, Some(pts));
    self
}

/// Compare char with last key pressed
pub fn iskey (&self, c:char) -> bool {
    match self.key {
        Some(k) => k==c,
        _ => false
    }
}

pub fn key (&mut self) -> Option<char> {
    let ret = self.key;
    self.key = None;
    ret
}

} // impl Plotter

fn _render (
    this: &mut Plotter,
    hmo: Option<&HashMapDeterministic>
) {
    //this.key = None; // Clear last keypressed
    // Plot either the internal or an external hashmap
    let hm = hmo.unwrap_or(&this.hm);
    if 0 == hm.len() { return } // No pixels, so skip rendering
    let mut eventrender :Option<Event> = None;
    while let Some(event) = this.pwin.next() {
        match event {
            Event::Loop(Loop::Render(_args)) => {
                eventrender = Some(event); // Handle rendering after this loop
                break
            },
            Event::Input(
                Input::Button(
                    ButtonArgs{ state, button:Button::Keyboard(k), scancode:_ }
                ),
                _optiontimestamp) => {
                if state == ButtonState::Press {
                    this.key = Some(k as u8 as char)
                }
            },
            _ => ()
        }
    }
    if eventrender.is_none() { return }
    let colors = &this.colors;
    this.pwin.draw_2d(
        &eventrender.unwrap(),
        | _c:Context,  g:&mut G2d,  _d:&mut GfxDevice | {
            let bb = bounding_box(&hm);
            // The transform matrix to fit all points in window
            let bounding_box_xform =
                [[2.0/bb.width, 0.0,           (bb.x.max + bb.x.min + 0.0) / -bb.width ],
                 [0.0,          2.0/bb.height, (bb.y.max + bb.y.min + 0.0) / -bb.height ]];
            clear(*colors.get(&0).unwrap_or(&[0.0, 0.0, 0.0, 1.0]), g);
            for ((x, y), c) in hm {
                rectangle(
                    *colors.get(c).unwrap_or(&[1.0, 0.7, 0.5, 1.0]),
                    [*x as f64, *y as f64, 1.0, 1.0], // x,y, w,h
                    bounding_box_xform,
                    g);
            }
        }
    );
    // Acquire the PostRender event which finally renders the images
    this.pwin.next();
} // fn _render

////////////////////////////////////////////////////////////////////////////////
// Return bounding box for x,y coordinates in the hashmap of points.

struct MinMax { min:f64, max:f64 }
struct BoundingBox { x:MinMax, y:MinMax, width:f64, height:f64 }

fn bounding_box (hm: &HashMapDeterministic) -> BoundingBox {
    let (xmin, mut xmax, ymin, mut ymax) =
        hm.iter().fold(
            (std::i32::MAX, std::i32::MIN, std::i32::MAX, std::i32::MIN),
            | mut r, ((x,y),_) | {
                if *x  < r.0 { r.0 = *x };
                if r.1 < *x  { r.1 = *x };
                if *y  < r.2 { r.2 = *y };
                if r.3 < *y  { r.3 = *y };
                r
            }
        );
    xmax += 1;
    ymax += 1;
    BoundingBox{
         x      :MinMax{min:xmin as f64, max:xmax as f64},
         y      :MinMax{min:ymin as f64, max:ymax as f64},
         width  :(xmax-xmin) as f64,
         height :(ymax-ymin) as f64
    }
}