#![allow(dead_code, unused_variables, non_snake_case)]

// External
use std::collections::{HashMap, HashSet};
use ::std::{fs, thread};
use ::std::ops::{Range};
use ::piston_window::*;
// Local
use ::util::{self};
use ::term::{Term};

fn fun_split_helper (v :&mut Vec<i32>) {
    v[0] = 200;
    v.push(9);
    println!("v={:?}", v);
}

/// fun_split
/// Play around with Rust basics that split a string into substring.
pub fn fun_split() {
    ::std::println!("== {}:{} ::{}::fun_split() ====", std::file!(), core::line!(), core::module_path!());
    println!(
        "[fun_string] {:?}",
        String::from("long live  donut ").split(" ").map(|s| s.trim()).collect::<Vec<&str>>()
    );
    let v = &mut vec![1]; // A vector               [1]
    println!("v={:?}", v);
    fun_split_helper(v);
    println!("v={:?}", v);

    v[0] = 69; // Mutate first position  [69]
    println!("v={:?}", v);
    fun_split_helper(v);
    println!("v={:?}", v);

    let f = &mut v[0]; // Ref to 1st position
    *f = 42; // Mutate first position  [42]
    println!("v={:?}", v);

    v.iter_mut().for_each( |x| *x = *x * 1000 );
    v.iter_mut().enumerate().for_each( |(i, x)| *x = *x + i as i32 );

    println!("v={:?}", v);
}

fn fun_tuples() {
    let s = (11, { println!("fun_tuples"); 22 });
    println!( "Tuple.? = {:?}", loop { break if true { s.0 } else { s.1 }; });
}

type Vi32 = Vec<i32>;

fn fun_map() {
    let r = (17..=20).collect::<Vi32>();
    //println!("{:?}", &r); // Can't move this after the for loop
    for i in &r { println!("{:?}", i); }
    println!("{:?}", r);
    let v = 5;
    println!("map {:?}",  (0..=v).map(|x| x / 2_i32).collect::<Vec<i32>>()); // type std::ops::RangeInclusive
    println!("iter {:?}", [0, 1, 2, 3].iter().map(|x| x * x).collect::<Vec<i32>>()); // type [i32; 2]
}

fn fun_write_non_block(term: &mut Term) {
    term.terminalraw();
    term.termnonblock();
    use std::io::Write;
    loop {
        match std::io::stdout().write(b"abcdefghijklmnopqrstuvwxyz") {
            // .as_bytes()
            Ok(o) => {
                println!(" Ok{}", o);
                if o != 26 {
                    break;
                }
            }
            Err(e) => {
                util::flush();
                println!("Err{}", e);
                util::sleep(1000);
                util::flush();
            }
        }
    }
    //||->Result<(), &str> {Err("no")?;Ok(())}().unwrap_err().as_bytes());
    term.done();
}

fn fun_wait_q_press(term: &Term) {
    term.terminalraw();
    //term.termblock();
    let mut s: String = String::new();
    while s.len() != 1 || &s[0..1] != "q" {
        s = term.getc();
        println!("getc '{}' {}'", s, s.len());
        //if s.as_bytes()[0] as char == 'q' { break; }
        if 0 == s.len() {
            util::sleep(1000);
        }
    }
}

fn fun_thread(term: Term) {
    thread::spawn(move || {
        fun_wait_q_press(&term);
        println!("thread done.]");
    });
    let mut c = 10;
    let term = Term::new();
    while 0 < c {
        fun_wait_q_press(&term);
        println!("[c={}]", c);
        c -= 1;
        util::sleep(500);
    }
}

fn fun_walk_iter() {
    let mut w = util::Walk::new(&[0.0, 0.0], &[10.0, 2.0]);
    println!("{:?} ", w);
    println!("{:?}", w.next());
    println!("{:?}", w);
    println!("{:?}", w.collect::<Vec<[i32; 3]>>());
    //for l in w { print!("{:?}", l); }
    //for l in w { print!("{:?}", l); }
}

#[derive(Debug)]
struct Pair<T> { // TODO make this enum duh
  car :Option<Box<T>>,
  cdr :Option<Box<Pair<T>>>
}

impl<T> Pair<T> {
    pub fn new (car :T, cdr :Pair<T>) -> Self {
       self::Pair {
           car : Some(Box::new(car)),
           cdr : Some(Box::new(cdr)),
       }
    }

    pub fn Null () -> Self { self::Pair { car: None, cdr: None} }
}

fn parse_dat_pair (s: &str, y: u32, x: u32) -> Pair<(char, u32, u32)> {
    if 0 == s.len() {
        Pair::Null()
    } else {
        match s.as_bytes()[0] as char {
            ' ' => parse_dat_pair(&s[1..], y, x+1),
            '\n' => parse_dat_pair(&s[1..], y+1, 0),
            c => Pair::new((c, y, x), parse_dat_pair(&s[1..], y, x+1))
        }
    }
}


fn fun_read_file_pair() {
    let s: &str = &fs::read_to_string("data/ship.dat").unwrap();
    let p :&Pair<(char, u32, u32)> = &parse_dat_pair(s, 0, 0);
    println!("p       {:?}", p);
    println!("p.car   {:?}", p.car.as_ref().unwrap());
    println!("p.cadr  {:?}", p.cdr.as_ref().unwrap().car.as_ref().unwrap());
    println!("p.caddr {:?}", p.cdr.as_ref().unwrap().cdr.as_ref().unwrap().car.as_ref().unwrap());
}

type Cpoint = (char, f32, f32);

fn parse_dat (s: &str, y: u32, x: u32, max :Cpoint, vcp :&mut Vec<Cpoint>) -> Cpoint {
    if 0 != s.len() {
        match s.as_bytes()[0] as char {
            ' ' => parse_dat(&s[1..], y, x+1, max, vcp),
            '\n' => parse_dat(&s[1..], y+1, 0, max, vcp),
            c => {
                vcp.push((c, y as f32, x as f32));
                parse_dat(&s[1..], y, x+1, ('m', max.1.max(y as f32), max.2.max(x as f32)), vcp)
            }
        }
    } else {
        max
    }
}


pub fn fun_read_poly_file (filename : &str) -> Vec::<Cpoint> {
    let mut vcp = Vec::<Cpoint>::new();
    let mut max :Cpoint = (' ', 0.0, 0.0);
    match fs::read_to_string(filename) {
        Ok(filestr) => 
            max = parse_dat(
                &filestr, // File as a string
                0, // y file loc
                0, // x file loc
                ('m', 0.0, 0.0),
                &mut vcp),
        _ => {
            ::log::info!("File '{}' not found during polygon conversion.", filename);
            ::log::warn!("File '{}' not found during polygon conversion.", filename);
            ::log::error!("File '{}' not found during polygon conversion.", filename);
            // Default polygon hourglass shape.
            vcp.push(('0',  0f32,  0f32));
            vcp.push(('0',  0f32,  1f32));
            vcp.push(('0',  1f32,  0f32));
            vcp.push(('0',  1f32,  1f32));
            max = (' ', 1f32, 1f32)
        }
    }
    //println!("{:?} max = {:?}", core::module_path!(), max);
    vcp.sort_by_key( |k| k.0 );
    let hvcp = vcp.into_iter().map( |p| {
         //println!(" {:?}", p);
         (p.0,
          p.1 / max.1 * 2.0 - 1.0,
          p.2 / max.2 * 2.0 - 1.0)
    }).collect::<Vec::<Cpoint>>();
    //hvcp.into_iter().count();
    //println!(" {:?}", hvcp);
    hvcp
}

/// Create a random f64 number
pub fn r64(m: f32) -> f64 { ::rand::random::<f64>() * m as f64 }

////////////////////////////////////////////////////////////////////////////////
// Play with Graphics
//

/// Iterator for walking around a box
#[derive(Debug, Clone, Copy)]
struct MarchBox {
    width: i64, // Box Dimension
    height: i64,
    x: i64, // Current location
    y: i64,
    skip: i64, // Step
    dir: usize, // Current edge
}

impl Iterator for MarchBox {
    type Item = [f64; 2];
    fn next(self: &mut MarchBox) -> Option<Self::Item> {
        match self.dir {
            0 => self.x = self.x + self.skip,
            1 => self.y = self.y + self.skip,
            2 => self.x = self.x - self.skip,
            3 => self.y = self.y - self.skip,
            _ => (),
        }

        if self.width <= self.x {
            let s = self.x - self.width;
            self.x = self.width - 1;
            self.y = self.y + s;
            self.dir = (self.dir + 1) % 4;
        }
        else if self.x < 0 {
            let s = -self.x;
            self.x = 0;
            self.y = self.y - s;
            self.dir = (self.dir + 1) % 4;
        }
        else if self.height <= self.y {
            let s = self.y - self.height;
            self.y = self.height - 1;
            self.x = self.x - s;
            self.dir = (self.dir + 1) % 4;
        }
        else if self.y < 0 {
            let s = -self.y;
            self.y = 0;
            self.x = self.x + s;
            self.dir = (self.dir + 1) % 4;
        }

        Some([self.x as f64, self.y as f64])
    }
}

/// Create a Piston window and draw things on it.
pub fn fun_piston_walk() {
    let W: i64 = 800;
    let H: i64 = 700;
    let mut count: i32 = 0;
    let mut mb = MarchBox {
        width: W,
        height: H,
        x: 0,
        y: 0,
        skip: 11,
        dir: 0,
    };
    let mut pwindow: PistonWindow = WindowSettings::new("ASCIIRhOIDS", [W as u32, H as u32])
        .exit_on_esc(true)
        .decorated(true)
        .build()
        .unwrap();
    let mut kolor = [crate::r32(1.0), crate::r32(1.0), crate::r32(1.0), 1.0];
    let mut next = mb.next().unwrap();

    while let Some(event) = pwindow.next() {
        if event.text_args() != None && event.text_args().unwrap() == "q" { break; }
        if event.render_args() == None { continue; } // Skip any non-render event.
        pwindow.draw_2d(
            &event,
            |context: piston_window::Context,
             graphics,
             _device| {
                line(
                    kolor,
                    2.0,
                    [W as f64/2.0, H as f64/2.0, next[0], next[1]],
                    context.transform,
                    graphics,
                );
            },
        );
        //util::sleep(100);
        if 1 == count % 2 {
            kolor = [crate::r32(1.0), crate::r32(1.0), crate::r32(1.0), 1.0];
            next = mb.next().unwrap();
            //pwindow.set_title(format!("{:?}", kolor));
        }
        count = count + 1;
    }
}


pub fn fun_fizzbuzz() {
    'main: for i in 1..=30 {
        for p in [(15, "fizzbuzz"),
                  (5, "buzz"),
                  (3, "fizz")].iter()
        {
            if 0 == i % p.0 {
                 println!("{}", p.1);
                 continue 'main;
            }
        }
        println!("{}", i);
    }
}
////////////////////////////////////////////////////////////////////////////////
// Learning
//
use ::std::ops::{Mul, MulAssign};

#[derive(Debug)]
enum Mat { Mat4([f64; 2]) }

impl MulAssign<f64> for Mat {
    fn mul_assign(&mut self, s:f64) {
        let Mat::Mat4(m) = self;
        m[0] *= s;
        m[1] *= s;
    }
}

fn fun_overload() {
    let mut m = Mat::Mat4([1.0,2.0]);
    m *= 11.0;
    println!("{:?}", m);
}
////////////////////////////////////////////////////////////////////////////////

fn fun_goto (mut i: usize) -> usize {
 'a:loop {
     'b:loop{
            match i { 0=>break'b, _=>{i-=1;continue'a} }
        }
        break'a i;
    }
    .checked_add(0)
    .map( |e| {
        println!("Returning {}", e);
        e } )
    .unwrap()
}

fn fun_cloned() {
    ::std::println!("== {}:{} ::{}::fun_cloned() ====", std::file!(), core::line!(), core::module_path!());

    // Vector
    let v = vec!(1,2,3,4,5,6,7,8,9,10);

    // Sets
    type Set = HashSet<usize>;
    let g = v.iter().cloned().collect::<Set>();
    let h = v.iter().map( |e| *e).collect::<Set>();

    // Vector of sets
    type VecSet = Vec<HashSet<usize>>;
    let mut vh = VecSet::new();
    vh.push(g.iter().cloned().collect::<Set>());
    vh.push(h.iter().cloned().collect::<Set>());

    println!("{:?}", v);
    println!("{:?}", g);
    println!("{:?}", h);
    println!("{:?}", vh);
}

////////////////////////////////////////////////////////////////////////////////
pub fn main() {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    //self::fun_split();
    //fun_tuples();
    //fun_map();
    //let mut term = Term::new();
    //fun_write_non_block(&mut term);
    //fun_wait_q_press(&term);
    //fun_thread(&term);
    //fun_walk_iter();
    //fun_read_file_pair();
    //for e in fun_read_poly_file("data/ship.dat") { println!("{:?}", e); }
    //fun_piston_walk();
    //self::fun_fizzbuzz();
    //crate::fun::fun_overload();
    //println!("{:?}", fun_goto(5));
    fun_cloned();
}