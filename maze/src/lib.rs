use ::std::{thread};
use ::std::net::{TcpListener}; // TcpStream
use ::std::io::prelude::*; // OR use std::io::{Write, Read}
use ::util::{Prng};
use ::std::collections::{HashMap};

fn newloc (mut loc:(i32,i32), dir:u32, amt:i32) -> (i32,i32) {
    match dir%4 { 0 => loc.0 += amt, 1 => loc.1 += amt, 2 => loc.0 -= amt, 3 => loc.1 -= amt, _ => () };
    loc
}

fn locmove (loc: &mut (i32,i32), dir: u32) {
    match dir { 0 => loc.0 += 1, 1 => loc.1 += 1, 2 => loc.0 -= 1, 3 => loc.1 -= 1, _ => () };
}

fn locpeek (m: &::util::PlotterPoints, loc0: (i32,i32), d: u32) -> bool {
    // Something in our way?
    let loc = newloc(loc0, d, 1);
    if m.get(&loc).is_some() { return true }
    if m.get(&newloc(loc, d,   1)).is_some() { return true } // In front of new spot?
    if m.get(&newloc(loc, d+3, 1)).is_some() { return true } // Right of new spot?
    if m.get(&newloc(loc, d+1, 1)).is_some() { return true } // Left of new spot?

    let loc = newloc(loc, d, 1);
    if m.get(&newloc(loc, d+3, 1)).is_some() { return true } // Diagonal right of new spot
    if m.get(&newloc(loc, d+1, 1)).is_some() { return true } // Diagonal left of new spot
    false
}

fn fun_maze(prng: &mut Prng, mut count: usize) -> util::Plotter {
    let mut pltr = ::util::Plotter::new(); 
    let mut manual = false;
    let mut loc = (0, 0);
    let mut k :i32 = 1;  // color index
    let mut kk :f32 = 0.1;
    let mut dir = 0; // direction
    pltr.insert(loc.0, loc.1, 2); // Add pixel to hash
    loop {
        count -= 1;
        if !true { match pltr.render().key {
            Some('q') => break,
            Some(' ') => manual = false,
            Some('l') => { dir=0; manual = true; }
            Some('k') => { dir=1; manual = true; }
            Some('h') => { dir=2; manual = true; }
            Some('j') => { dir=3; manual = true; }
                    _ => if manual { continue }
        } }

        let choosenewcolor = false;
        if !manual {
            let mut retry = 1;
            loop {
                // peek
                dir = prng.u32(4);
                if !locpeek(&pltr.hm, loc, dir) { break } // Can move in this direction
                if retry < 1 { // Choose a new location in the maze if we can't walk in new direction
                    loc = *pltr.hm.iter().nth( prng.usize(pltr.hm.len()) ).unwrap().0;
                    //choosenewcolor = true;
                    continue
                }
                retry -= 1;
            }
        }

        if choosenewcolor {
            k += 1;
            //pltr.color(k, [rf32()*0.5+0.5, rf32()*0.5+0.5, rf32()*0.5+0.5, 1.0]);
            //pltr.color(k, [kk, 0.0, 0.0, 1.0]);
            //pltr.color(k, [prng.f32(0.8)+0.2, prng.f32(0.8)+0.2, prng.f32(0.8)+0.2, 1.0]);
            kk += 0.001;
            if 1.0 < kk { kk = 0.2 }
        }
        locmove(&mut loc, dir);
        if count <= 0 {
            pltr.insert(loc.0, loc.1, 15);
            break
        }
        pltr.insert(loc.0, loc.1, k);
        //println!("pts{} clr{}", pltr.hm.len(), pltr.colors.len());
    }
    pltr
}

/// Walker
struct Walker {
    pub hm: util::PlotterPoints,
    pub loc: (i32, i32)
}

impl Walker {
    pub fn new (hm: util::PlotterPoints, loc: (i32, i32)) -> Walker { Walker{hm, loc} }
    pub fn walk (&mut self, dir:u8) -> u8 {
        let d = match dir as char {
            'e'|'E'|'C' => 0,
            'n'|'N'|'A' => 1,
            'w'|'W'|'D' => 2,
            's'|'S'|'B' => 3,
            _ => 42};
        if 3 < d { return b'n' }
        let loc2 = newloc(self.loc, d, 1);
        if let Some(k) = self.hm.get(&loc2) {
            self.loc = loc2;
            match *k {
                15 => b'e',
                2 => b's',
                _ => b'y'
            }
        } else {
            b'n'
        }
    }
    pub fn plot (&self) {
        let size = 50;
        for y in (0..size).rev() {
            for x in 0..size {
                if x==size/2 && y == size/2
                    { print!("\x1b[1;34m@\x1b[0m") }
                else if let Some(c) = self.hm.get( &(x+self.loc.0-size/2, y+self.loc.1-size/2) )
                    { print!("\x1b[3{:?}m#\x1b[0m", c) }
                else
                    { print!(".") }
            }
            println!("");
        }
    }

}

fn server (maze: &mut util::Plotter) {
    if let Ok(listener) = TcpListener::bind("127.0.0.1:8888") { // TcpListener
        for stream in listener.incoming() { // BLOCKING
            let points =
                maze.hm.iter()
                .fold(
                    HashMap::with_hasher(util::DeterministicHasher{}),
                    |mut r,(k,v)| {
                        r.insert(*k,*v);
                        r
                    });
            let mut walker = Walker::new(points, (0, 0));
            thread::spawn( move || {
                let mut buff :[u8;1] = [0; 1];
                if let Ok(mut stream) = stream { // TcpStream
                    loop {
                        walker.plot();
                        let mut count;
                        if let Ok(c) = stream.read(&mut buff) {
                            print!("\x1b[32m{:?}\x1b[0m", std::str::from_utf8(&buff[0..1]).unwrap() );
                            count = c;
                        } else {
                            print!("\x1b[42;30mERR\x1b[0m");
                            break;
                        }
                        if 0==count || buff[0] == b'q' { break }

                        buff[0] = walker.walk(buff[0]);

                        if let Ok(c) = stream.write(&buff[0..1]) {
                            println!("\x1b[33m{:?}\x1b[0m", &buff[0..1] );
                            count = c;
                        } else {
                            print!("\x1b[43;30mERR\x1b[0m");
                            break;
                        }
                        if 0==count { break }
                    }
                    match stream.shutdown(std::net::Shutdown::Both) {
                        o => println!("\x1b[31mshutdown:{:?}\x1b[0m", o),
                    }
                }
            });
        }
    }
}

pub fn main () {
    // 6 100
    // 4 300
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    let mut prng = Prng::new(6);
    let mut maze = fun_maze(&mut prng, 100);
    //maze.render();
    server(&mut maze);
}