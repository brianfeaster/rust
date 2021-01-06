use ::std::{thread}; //io::Result;
use ::std::net::{TcpListener, TcpStream};
use ::std::{io::{self, prelude::*}}; // OR use std::io::{Write, Read}
use ::std::collections::{HashMap};
use ::util::{Prng, HashMapDeterministic, hashmapdeterministicnew};

////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////

fn newloc (mut loc:(i32,i32), dir:u32, amt:i32) -> (i32,i32) {
    match dir%4 {
        0 => loc.0 += amt,
        1 => loc.1 += amt,
        2 => loc.0 -= amt,
        3 => loc.1 -= amt,
        _ => ()
    };
    loc
}

fn locmove (loc: &mut (i32,i32), dir: u32) {
    match dir { 0 => loc.0 += 1, 1 => loc.1 += 1, 2 => loc.0 -= 1, 3 => loc.1 -= 1, _ => () };
}

fn locpeek (m: &HashMapDeterministic, loc0: (i32,i32), d: u32) -> bool {
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

pub fn maze_create(
    prng        :&mut Prng,
    mut count   :usize,
    bplotter    :&mut Box::<dyn FnMut(&HashMapDeterministic)>
) -> HashMapDeterministic {
    let mut hm = hashmapdeterministicnew();
    let manual = false;
    let mut loc = (0, 0);
    let mut k :i32 = 1;  // color index
    let mut kk :f32 = 0.1;
    let mut dir = 0; // direction
    hm.insert(loc, 2); // Add pixel to hash
    loop {
        count -= 1;
        /*
        if !true { match pltr.render().key {
            Some('q') => break,
            Some(' ') => manual = false,
            Some('l') => { dir=0; manual = true; }
            Some('k') => { dir=1; manual = true; }
            Some('h') => { dir=2; manual = true; }
            Some('j') => { dir=3; manual = true; }
                    _ => if manual { continue }
        } }
        */

        let choosenewcolor = false;
        if !manual {
            let mut retry = 1;
            loop {
                // peek
                dir = prng.u32(4);
                if !locpeek(&hm, loc, dir) { break } // Can move in this direction
                if retry < 1 { // Choose a new location in the maze if we can't walk in new direction
                    loc = *hm.iter().nth( prng.usize(hm.len()) ).unwrap().0;
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
            hm.insert(loc, 14);
            break
        }
        hm.insert(loc, k);
        bplotter(&hm);
    }
    bplotter(&hm);
    hm
}

/// Walker
type Points = HashMap<(i32,i32),i32>;

struct MazeWalker {
    pub hm: Points,
    pub loc: (i32, i32)
}

impl MazeWalker {

pub fn new (maze: &HashMapDeterministic, loc: (i32, i32)) -> MazeWalker {
    let points :Points = maze.iter().map(|(k,v)|(*k,*v)).collect();
    MazeWalker{ hm:points, loc:loc }
}

/// Update, if direction is valid, location and point (maze cell) color
/// Return 'n' when illegal, 'y' 's' or 'e' when legal
pub fn walk (&mut self, dir:u8) -> Option<u8> {
    let d = match dir {
        b'e'|b'E'|b'C'|0 => 0,
        b'n'|b'N'|b'A'|1 => 1,
        b'w'|b'W'|b'D'|2 => 2,
        b's'|b'S'|b'B'|3 => 3,
        _ => 42};
    if 3 < d { return None } // Only 4 directions are valid
    let loc2 = newloc(self.loc, d, 1);
    let k = self.hm.get(&loc2)?;
    self.loc = loc2;
    Some(match *k {
        14 => b'e',
        2  =>  b's',
        _  => { self.hm.insert(loc2, 3); b'y' }
    })
}
pub fn plot (&self) {
    let size = 15;
    print!(" \x1b7\x1b[15A\r"); // back, save, move
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
    print!("\x1b8"); // restore, forward, space
    util::flush();
}

fn walkloop (
    &mut self,
    stream: &mut TcpStream
) -> io::Result<&'static str> {
    let mut buff :[u8;1] = [0; 1];
    let mut byte_count;
    print!("{}", "\n".repeat(15));
    loop {
        self.plot(); // Render maze

        byte_count = stream.read(&mut buff)?; // Read a byte from player
        if 0==byte_count { return Ok("read 0 bytes expected 1") }
        if buff[0]==b'q' { return Ok("read q") }

        let result_byte = self.walk(buff[0]).unwrap_or(b'n'); // Update maze walker's state
        let result_color = match result_byte { b'y'=>1, b's'=>2, b'e'=>3, _ =>7 };

        let token = format!("{:?}", buff[0] as char);
        print!("\x1b[3{}m{}\x1b[0m", result_color, &token[1..token.len()-1] ); // Log user's byte with result color

        buff[0] = result_byte;
        byte_count = stream.write(&buff[0..1])?; // Send a byte to player
        if 0==byte_count { return Ok("wrote 0 bytes expected 1") }
    }
}

} // impl MazeWalker

pub fn server (
    maze    :&mut HashMapDeterministic
) -> io::Result<&'static str> {
    let listener :TcpListener = TcpListener::bind("127.0.0.1:1777")?;
    for stream in listener.incoming() { // This blocks
        let mut stream = stream?;
        let mut walker = MazeWalker::new(&maze, (0,0));
        thread::spawn( move || {
            println!(" Maze Walker Loop: {:?}", walker.walkloop(&mut stream));
            println!(" Stream shutdown: {:?}", stream.shutdown(std::net::Shutdown::Both));
        });
    }
    Ok("Server iterator finite expected infinite")
}

pub fn start (
    seed: u64,
    size: usize,
    mut bplotter: Box::<dyn FnMut(&HashMapDeterministic)>
) {
    ::std::println!("== {}:{} ::{}::start() ====", std::file!(), core::line!(), core::module_path!());
    let mut prng = Prng::new(seed); // Random number generator
    let mut maze = maze_create(&mut prng, size, &mut bplotter);
    let error = server(&mut maze);
    println!("{:?}", error);
}