// External
use ::std::{fs, thread, fmt};
use ::std::collections::{HashMap, HashSet};
use ::std::io::{self, prelude::*}; // OR use std::io::{Write, Read}
use ::std::net::{TcpListener, TcpStream};
use ::std::ops::{Range};
use ::log::*;
use ::serde::{Serialize, Deserialize};
use ::serde_json::{self as sj, Value, from_str, to_string_pretty};
use ::piston_window::*;
// Local
use ::utils::*;
use ::term::{Term};

fn fun_split_helper (v :&mut Vec<i32>) {
    v[0] = 200;
    v.push(9);
    println!("v={:?}", v);
}

/// fun_split
/// Play around with Rust basics that split a string into substring.
pub fn fun_split() {
    println!("== {}:{} ::{}::fun_split() ====", std::file!(), core::line!(), core::module_path!());
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

////////////////////////////////////////////////////////////////////////////////

fn fun_tuples() {
    println!("== {}:{} ::{}::fun_tuples() ====", std::file!(), core::line!(), core::module_path!());
    let s = (11, { println!("fun_tuples"); 22 });
    println!( "Tuple.? = {:?}", loop { break if true { s.0 } else { s.1 }; });
}

////////////////////////////////////////////////////////////////////////////////

type Vi32 = Vec<i32>;

fn fun_map() {
    println!("== {}:{} ::{}::fun_map() ====", std::file!(), core::line!(), core::module_path!());
    let r = (17..=20).collect::<Vi32>();
    //println!("{:?}", &r); // Can't move this after the for loop
    for i in &r { println!("{:?}", i); }
    println!("{:?}", r);
    let v = 5;
    println!("map {:?}",  (0..=v).map(|x| x / 2_i32).collect::<Vec<i32>>()); // type std::ops::RangeInclusive
    println!("iter {:?}", [0, 1, 2, 3].iter().map(|x| x * x).collect::<Vec<i32>>()); // type [i32; 2]
}

////////////////////////////////////////////////////////////////////////////////

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
                flush();
                println!("Err{}", e);
                sleep(1000);
                flush();
            }
        }
    }
    //||->Result<(), &str> {Err("no")?;Ok(())}().unwrap_err().as_bytes());
    term.done();
}

////////////////////////////////////////////////////////////////////////////////

fn fun_wait_q_press(term: &Term) {
    term.terminalraw();
    //term.termblock();
    let mut s: String = String::new();
    while s.len() != 1 || &s[0..1] != "q" {
        s = term.getc();
        println!("getc '{}' {}'", s, s.len());
        //if s.as_bytes()[0] as char == 'q' { break; }
        if 0 == s.len() {
            sleep(1000);
        }
    }
    term.done();
}

////////////////////////////////////////////////////////////////////////////////

fn fun_thread(term: Term) {
    // The thread that read the keyboard in the background never ends...
    // so gets upset when it's parent dies.
    thread::spawn(move || {
        fun_wait_q_press(&term);
        println!("...thread done waiting.");
        term.done();
    });
    let mut c = 10;
    let term = Term::new();
    while 0 < c {
        fun_wait_q_press(&term);
        println!("main got 'q' {}/10", c);
        c -= 1;
        sleep(500);
    }
}

////////////////////////////////////////////////////////////////////////////////

fn fun_walk_iter() {
    let mut w = Walk::new(&[0.0, 0.0], &[10.0, 2.0]);
    println!("{:?} ", w);
    println!("{:?}", w.next());
    println!("{:?}", w);
    println!("{:?}", w.collect::<Vec<[i32; 3]>>());
    //for l in w { print!("{:?}", l); }
    //for l in w { print!("{:?}", l); }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct PairCons<T> { // TODO make this enum duh
  car :Option<Box<T>>,
  cdr :Option<Box<PairCons<T>>>
}

impl<T> PairCons<T> {
    pub fn new (car :T, cdr :PairCons<T>) -> Self {
       self::PairCons {
           car : Some(Box::new(car)),
           cdr : Some(Box::new(cdr)),
       }
    }

    pub fn Null () -> Self { self::PairCons { car: None, cdr: None} }
}

fn parse_dat_pair (s: &str, y: u32, x: u32) -> PairCons<(char, u32, u32)> {
    if 0 == s.len() {
        PairCons::Null()
    } else {
        match s.as_bytes()[0] as char {
            ' ' => parse_dat_pair(&s[1..], y, x+1),
            '\n' => parse_dat_pair(&s[1..], y+1, 0),
            c => PairCons::new((c, y, x), parse_dat_pair(&s[1..], y, x+1))
        }
    }
}

fn fun_read_file_pair() {
    let s: &str = &fs::read_to_string("data/ship.dat").unwrap();
    let p :&PairCons<(char, u32, u32)> = &parse_dat_pair(s, 0, 0);
    println!("p       {:?}", p);
    println!("p.car   {:?}", p.car.as_ref().unwrap());
    println!("p.cadr  {:?}", p.cdr.as_ref().unwrap().car.as_ref().unwrap());
    println!("p.caddr {:?}", p.cdr.as_ref().unwrap().cdr.as_ref().unwrap().car.as_ref().unwrap());
}

////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////

fn fun_log () {
    ::pretty_env_logger::init();
    ::std::println!("== {}:{} ::{}::fun_log() ====", std::file!(), core::line!(), core::module_path!());
    ::log::trace!("A fun info log {:?} RUST_LOG=trace", 1);
    ::log::debug!("A fun info log {:?} RUST_LOG=debug", 1);
    ::log::info!("A fun info log {:?} RUST_LOG=info", 1);
    ::log::warn!("A fun warn log {:?} RUST_LOG=warn", 2);
    ::log::error!("A fun error log {:?} RUST_LOG=error", 3);
}

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
    let mut kolor = [rf32(1.0), rf32(1.0), rf32(1.0), 1.0];
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
        //sleep(100);
        if 1 == count % 2 {
            kolor = [rf32(1.0), rf32(1.0), rf32(1.0), 1.0];
            next = mb.next().unwrap();
            //pwindow.set_title(format!("{:?}", kolor));
        }
        count = count + 1;
    }
}

////////////////////////////////////////////////////////////////////////////////

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

fn fun_cloned() {
    ::std::println!("== {}:{} ::{}::fun_cloned() ====", std::file!(), core::line!(), core::module_path!());
    // Vector
    let v = vec!(1,2,3,4,5,6,7,8,9,10);
    // Sets
    type Set = HashSet<usize>;
    let g = v.iter().cloned().collect::<Set>();
    let h = v.iter().map( |e| *e+1000).collect::<Set>();
    // Vector of sets
    type VecSet = Vec<HashSet<usize>>;
    let mut vh = VecSet::new();
    vh.push(g.iter().cloned().collect::<Set>());
    vh.push(h.iter().cloned().collect::<Set>());

    println!("v {:?}", v);
    println!(" g cloned of v {:?}", g);
    println!(" h map over v  {:?}", h);
    println!("v {:?}\n", v);

    println!("vh {:?}", vh);
}

////////////////////////////////////////////////////////////////////////////////

fn fun_emojis () {
    println!("map {:?}",
        ('🐘' ..= '🐷') // std::ops::RangeInclusive
        .map(|x| (|x| x)(x))
        .collect::<Vec<char>>()
    );
}

////////////////////////////////////////////////////////////////////////////////
// Play with json
//

#[derive(Debug)]
pub enum MyError {
    IoError(std::io::Error),
    JsonError(json::Error),
    SerdeJsonError(serde_json::Error)
}

impl From<std::io::Error>     for MyError { fn from(error: std::io::Error)    -> Self { MyError::IoError(error) }   }
impl From<json::Error>        for MyError { fn from(error: json::Error)       -> Self { MyError::JsonError(error) } }
impl From<serde_json::Error>  for MyError { fn from(error: serde_json::Error) -> Self { MyError::SerdeJsonError(error) } }

fn mainJson () -> Result<usize, MyError> {
    Ok(
        json::parse(&fs::read_to_string("products.json")?)?
        ["treats"]
        .members()
        .map( |e| println!("{}", e["name"].pretty(1)) )
        .count()
    )
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug)]
struct BulkPricing {
    amount : i32,
    totalPrice : f32
}

#[derive(Serialize, Deserialize, Debug)]
struct Treat {
    id: i32,
    name: String,
    imageURL: String,
    price: f32,
    bulkPricing: Option<BulkPricing>
}

#[derive(Serialize, Deserialize, Debug)]
struct Products {
    treats :Vec<Treat>
}

fn mainJsonSerdes () -> Result<usize, MyError> {
    let v :Products //HashMap<String, Vec<sj::Value>>
        = sj::from_str(&fs::read_to_string("products.json")?)?;
    Ok(
        v.treats
        .iter()
        .map( |e| println!("{}", sj::to_string_pretty(&e.name).unwrap()))
        .count()
    )
}

////////////////////////////////////////////////////////////////////////////////

fn fun_json () {
    println!("== {}:{} ::{}::fun_json() ====", std::file!(), core::line!(), core::module_path!());
    println!("{:?}", mainJson());
    println!("!!! {:?}", mainJsonSerdes());
}

////////////////////////////////////////////////////////////////////////////////

fn fun_goto (mut i: usize) -> usize {
    println!("== {}:{} ::{}::fun_goto() ====", std::file!(), core::line!(), core::module_path!());
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

////////////////////////////////////////////////////////////////////////////////
// Float rounding testng

use std::mem::transmute;

fn round(f: f64) -> f64 {
    //(unsafe { transmute::<u64, f64>(transmute::<f64, u64>(f) + 1) } * 100.0).round() / 100.0
    unsafe { transmute::<u64, f64>(transmute::<f64, u64>(f) + 0) }
}

fn fun_checkfloats() {
    let mut errors = 0;
    let mut i = 000000_000i64;
    while i <=  999999_999i64 && errors < 10 {
        let n = i / 1000i64;
        let d = i % 1000i64;

        // Fixed point string
        let s = format!("{}.{:03}", n, d);
        // Fixed point rounded string
        let sr = {
            let mut n = n;
            let mut d = if d % 10 < 5 { d / 10 } else { d / 10 + 1 };
            if 100 <= d {
                n += 1;
                d -= 100;
            }
            format!("{}.{:02}", n, d)
        };

        // Float
        let f: f64 = s.parse::<f64>().map_err( |e| { println!("{:?} {:?}", e, s); e }).unwrap();
        // Float rounded string
        let fr = format!("{:.2}", round(f));

        if sr != fr {
            errors += 1;
            println!("{} '{}'/'{}'  {}/'{}'  {:.60}", i, s, sr, f, fr, f);
        }

        i += 1;
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn main() {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    //self::fun_split();
    //super::fun::fun_tuples();
    //crate::fun::fun_map();
    //let mut term = Term::new();
    //fun_write_non_block(&mut term);
    //fun_wait_q_press(&term);
    //fun_thread(term);
    //fun_walk_iter();
    //fun_read_file_pair();
    //for e in fun_read_poly_file("data/ship.dat") { println!("{:?}", e); }
    //fun_log();
    //fun_piston_walk();
    //fun_fizzbuzz();
    //fun_overload();
    //fun_cloned();
    //fun_emojis();
    //fun_json();
    fun_checkfloats();
    //println!("{:?}", fun_goto(5));
}