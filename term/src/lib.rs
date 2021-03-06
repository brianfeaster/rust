#![allow(dead_code, non_snake_case, unused_assignments)]

use ::utils;
use ::std::fmt;
use ::std::io::{stdin, stdout, Read, Write};
use ::std::sync::mpsc::{channel, Receiver, Sender};
use ::std::thread;
use piston_window::*;
use ::libc::{termios};

/// Implementation for "Pretty Printing" an object.
pub trait PrettyPrint {
    fn pp(&self);
}

/// Terminal Abstaction
pub struct Term {
    cols: usize,
    rows: usize,
    count: usize,
    original_termios: termios,
    original_fcntl: libc::c_int,
    keyin : Sender<String>,
    keyout : Receiver<String>,
    key : Receiver<String>,
}

impl Term {

    // Public in case you wish to force the terminal size
    pub fn termsizeset(self: &mut Term, cols: usize, rows: usize)  -> bool {
        let resized = self.cols != cols || self.rows != rows;
        self.cols  = cols;
        self.rows  = rows;
        self.count = self.cols * self.rows;
        return resized;
    }

    pub fn termsize(&mut self) -> bool {
        let winsize = ::libc::winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        unsafe {
            libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &winsize);
        }
        return self.termsizeset(winsize.ws_col as usize, winsize.ws_row as usize);
    }

    /*
    fn termsetup(&mut self) {
        unsafe {
            // Register WINCH signal's handler
            match register(SIGWINCH, ||{Term::termsize(self);}) {
                Ok(_) => { },
                Err(err) => {println!("ERROR:: register(SIGWINCH, ...) {}", err);}
            }
        }
    }
    */

    /// Set non-blocking mode on STDIN.
    pub fn termnonblock(&self) -> &Self {
        unsafe {
            utils::flush();
            libc::fcntl(0, libc::F_SETFL, &self.original_fcntl | libc::O_NONBLOCK);
        }
        return self;
    }

    pub fn termblock(&self) -> &Self {
        unsafe {
            utils::flush();
            libc::fcntl(0, libc::F_SETFL, &self.original_fcntl);
        }
        return self;
    }

    pub fn terminalraw(&self) -> &Self {
        unsafe {
            let mut bits = self.original_termios;
            libc::cfmakeraw(& mut bits);
            bits.c_oflag |= libc::OPOST; // OPOST (revert insert \r before \n behavior)
            libc::tcsetattr(0, libc::TCSANOW, &bits);
        }
        return self;
    }

    pub fn cursoroff(&self) -> &Self {
        print!("\x1b[?25l");
        return self;
    }

    pub fn cols (self: &Term) -> usize { return self.cols; }
    pub fn rows (self: &Term) -> usize { return self.rows; }
    pub fn count (self: &Term) -> usize { return self.count; }

    /// Blocking read from terminal.  Unbuffered if terminalraw() called.
    fn getc_actual () -> String {
        let mut buffer = [0; 256];
        let mut count = 0;
        match stdin().read(&mut buffer) {
            Ok(result) => { count = result; },
            Err(_msg) => ()
        }
        return std::str::from_utf8(&buffer[0..count]).expect("utf8 issue").to_string();
    }

    pub fn pushc (&self, s :String) {
        self.keyin.send(s).expect("Unable to send on keyboard buffer channel");
    }

    pub fn getc (self :& Term) -> String {
        let mut ss : String = String::new();
        loop {
            match self.key.try_recv() {
                Ok(s) => {ss = ss + &s; },
                Err(_e) =>  { break }
            };
        }
        loop {
            match self.keyout.try_recv() {
                Ok(s) => {ss = ss + &s; },
                Err(_e) =>  { break }
            };
        }
        return ss;
    }

    pub fn new () -> Self {
        let (tx, rx) = channel();
        let (txin, rxin) = channel();
        let mut term = Term{
            cols: 0,
            rows: 0,
            count: 0,
            original_termios: termios {
                c_iflag: 0,
                c_oflag: 0,
                c_cflag: 0,
                c_lflag: 0,
                #[cfg(target_os = "linux")] c_line: 0,
                #[cfg(target_os = "linux")] c_cc: [0; 32],
                #[cfg(target_os = "macos")] c_cc: [0; 20],
                c_ispeed: 0,
                c_ospeed: 0 },
            original_fcntl: 0,
            keyin: txin,
            keyout: rxin,
            key: rx
        };
        unsafe {
            libc::tcgetattr(0, &mut term.original_termios);
            term.original_fcntl = libc::fcntl(0, libc::F_GETFL);
        }
        thread::spawn(move || {
            loop {
                let s :String = Term::getc_actual(); // Blocking read.
                if s.is_empty() {
                    eprintln!("::term::Term::new - thread Term::getc_actual is empty...looping...");
                    utils::sleep(50);
                } else {
                  //println!("'TERM{} {}'", s, s.len());
                  //if s.as_bytes()[0] as char == 'Q' { break; }
                  tx.send(s.clone()).unwrap_or_else(|e| println!("ERROR: Term is unable to forward keyboard input to tx.send({:?}) {:?}", s, e));
                }
            }
        });
        return term;
    }

    pub fn done (self :& Term) {
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &self.original_termios);
            //self.termblock();
        }
        print!("\x1b[0m"); // Reset color
        print!("\x1b[?25h"); // Enable Cursor
        //print!("\x1b[{}H", self.rows); // Cursor to screen bottom
        utils::flush();
    }

} // impl Term

impl PrettyPrint for Term {
    fn pp(&self) {
        println!("{:?}", self);
    }
}

impl fmt::Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Term")
         .field("cols", &self.cols)
         .field("rows",  &self.rows)
         .field("count", &self.count) // &[]
         .field("c_iflag", &self.original_termios.c_iflag)
         .field("c_oflag", &self.original_termios.c_oflag)
         .field("c_cflag", &self.original_termios.c_cflag)
         .field("c_lflag", &self.original_termios.c_lflag)
         .field("c_cc",  &self.original_termios.c_cc)
         .field("c_ispeed", &self.original_termios.c_ispeed)
         .field("c_ospeed", &self.original_termios.c_ospeed)
         .field("c_int", &self.original_fcntl)
         .finish()
    }
}




/// Terminal Frame Buffer - A 3D vector abstraction for the terminal.
///
#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    pub ch: char,
    pub bg: i32,
    pub fg: i32,
    pub tick: usize
}

const GLYPH_NONE :Glyph = Glyph{ch:'\0', bg:0, fg:0, tick:0};
const GLYPH_BLANK :Glyph = Glyph{ch:' ', bg:0, fg:0, tick:0};

#[derive(Debug)]
pub struct Tbuff {
    buff: Vec<Glyph>,
    pub term: Term,
    tick: usize
}

impl Tbuff {

    pub fn getc (self :& Tbuff) -> String {
       self.term.getc()
    }

    pub fn pushc (self :& Tbuff, s:String) {
       self.term.pushc(s);
    }

    pub fn cols  (self :& Tbuff) -> usize { self.term.cols() }
    pub fn rows  (self :& Tbuff) -> usize { self.term.rows() }
    pub fn count (self :& Tbuff) -> usize { self.term.count() }
    pub fn dims (self :& Tbuff) -> (usize, usize, usize) { (self.cols(), self.rows(), self.count()) }

    pub fn reset (self: &mut Tbuff, tick : usize) -> &Self {
        self.tick = tick;
        let gen = tick & 1;
        if self.term.termsize() { // If the screen size changed reset our model of it.
            self.buff.resize((self.term.count() * 2) as usize, GLYPH_NONE);
            for i in 0..self.term.count() { // Set current and previous glyphs at each cell
                self.buff[(2*i + gen)] = GLYPH_BLANK;
                self.buff[(2*i + gen^1)] = GLYPH_NONE;
            }
        }
        return self;
    }

    pub fn reset_size_and_tick (self: &mut Tbuff, width :usize, height :usize) -> &Self {
        self.tick += 1;
        let gen = self.tick & 1;
        if self.term.termsizeset(width, height) {
            self.buff.resize((self.term.count() * 2) as usize, GLYPH_NONE);
            for i in 0..self.term.count() { // Set current and previous glyphs at each cell
                self.buff[(2*i + gen)] = GLYPH_BLANK;
                self.buff[(2*i + gen^1)] = GLYPH_NONE;
            }
        }
        return self;
    }

    pub fn set (self: &mut Tbuff, x:usize, y:usize, bg:i32, fg:i32, ch:char){
        let idx =
            (  x.rem_euclid(self.cols())
             + y.rem_euclid(self.rows()) * self.cols()) *
            2 + (self.tick&1); // index the right page
        self.buff[idx].ch = ch;
        self.buff[idx].bg = bg;
        self.buff[idx].fg = fg;
        self.buff[idx].tick = self.tick;
    }

    pub fn line (&mut self, vs1 :&[f32], vs2 :&[f32], _ch :char, color: i32) {
        let mut x = vs1[0] as i32;
        let mut y = vs1[1] as i32;
        for [xinc, yinc, c] in utils::Walk::new(vs1, vs2) {
            x += xinc;
            y += yinc;
           self.set(x as usize, y as usize, 0, color, c as u8 as char);
        }
    }
    
    /// Plot to non-terminal display device
    pub fn dumpPiston (
        self     :&mut Tbuff,
        context  :piston_window::Context,
        graphics :&mut G2d
    ) -> &Self {
        let ticknow = self.tick&1;
        let tickbak = ticknow ^ 1;
        let mut col=0;
        let mut row=0;
        //clear([0.0, 0.0, 0.0, 1.0], graphics);
        for i in 0..self.buff.len()/2 {
            // This glyph wasn't updated this tick.  So it's assumed to be a blank now.
            if self.buff[i*2+ticknow].tick != self.tick {
                self.buff[i*2+ticknow].ch = ' ';
                self.buff[i*2+ticknow].bg = 0;
                self.buff[i*2+ticknow].fg = 0;
            }
            let tnow = self.buff[i*2+ticknow].ch;
            let tbak = self.buff[i*2+tickbak].ch;
            if tnow != tbak {
                rectangle(
                    if tnow != ' ' { [ 0.0, 0.0, 1.0, 1.0 ] } else { [ 0.0, 0.0, 0.0, 1.0 ] },
                    [ col as f64 * 6.0, row as f64 * 6.0,
                    6.0,                6.0],
                    context.transform,
                    graphics);
            }
            /* else { // color unchanged
                rectangle(
                    [ 0.0, 0.1, 0.0, 1.0 ],
                    [ col as f64 * 6.0, row as f64 * 6.0,
                    6.0,                6.0],
                    context.transform,
                    graphics);
            } */
            col += 1;
            if col == self.term.cols() { col = 0; row += 1; } 
        }
        return self;
    } // Tbuff::dumpPiston

    /// Delta buffer -> terminal dumper
    /// If the glyph's tick matches current tick, then dump gyph (this glyph was updated)
    /// If the ticks don't match, assume a previous cell that should be erased
    ///   If already erased
    /// reset   tick=1
    /// [ ,-1]  [A,1]
    ///   init  renderi
    pub fn dump (self :&mut Tbuff) -> &Tbuff {
        let mut lbg: i32 = -1;
        let mut lfg: i32 = -1;
        let mut cb :[u8;4] = [0,0,0,0];
        if let Err(_e) = stdout().write("\x1b[H\x1b[0m".as_bytes()) {
           utils::flush();
        }
        let ticknow = self.tick & 1;
        let tickback = ticknow ^ 1;
        let mut glyph : Glyph = Glyph{ch:' ', bg:0, fg:0, tick:0};
        let mut col=0;
        let mut row=0;
        let mut rowlast=0;
        let mut skipped = 0;
        for i in 0..self.buff.len()/2 {
            // This glyph wasn't updated this tick.  So it's assumed to be a blank now.
            if self.buff[i*2+ticknow].tick != self.tick {
                self.buff[i*2+ticknow].ch = ' ';
                self.buff[i*2+ticknow].bg = 0;
                self.buff[i*2+ticknow].fg = 0;
            }
            if self.buff[i*2+ticknow].ch == self.buff[i*2+tickback].ch &&
               self.buff[i*2+ticknow].bg == self.buff[i*2+tickback].bg &&
               self.buff[i*2+ticknow].fg == self.buff[i*2+tickback].fg {
                skipped += 1;
            }  else {
                if skipped != 0 {
                    let m = if rowlast != row {
                        format!("\x1b[{};{}H", row+1, col+1)
                    } else {
                        format!("\x1b[{}C", skipped)
                    };
                    match stdout().write(m.as_bytes()) {
                      Ok(_o) => { }
                      Err(_e) => { }
                    }
                }
                skipped = 0;
                rowlast = row;
                // Current and last glyph don't match, so render.
                glyph = self.buff[i*2+ticknow];
                // Current and last glyph match, so skip
                if lfg != glyph.fg  && glyph.ch != ' '{
                    lfg = glyph.fg;
                    let bs = if lfg < 8 {
                        format!("\x1b[3{}m", lfg)
                    } else if lfg < 256 {
                        format!("\x1b[38;5;{}m", lfg)
                    } else {
                        format!("\x1b[48;2;{};{};{}m", lbg/65536, (lbg/256)%256, lbg%256)
                    };
                    match stdout().write(bs.as_bytes()) {
                      Ok(o) => { if o != bs.len() { utils::flush(); println!("{} != {}", bs.len(), o); utils::flush(); utils::sleep(5000); }},
                      Err(_e) => { utils::flush(); }
                    }
                }
                if lbg != glyph.bg {
                    lbg = glyph.bg;
                    let bs = if lbg < 8 { // 16 color
                        format!("\x1b[4{}m", lbg)
                    } else if  lbg < 256 { // 256 color
                        format!("\x1b[48;5;{}m", lbg)
                    } else { // 16M color
                        format!("\x1b[48;2;{};{};{}m", lbg/65536, (lbg/256)%256, lbg%256)
                    };
                    match stdout().write(bs.as_bytes()) {
                      Ok(o) => { if o != bs.len() { utils::flush(); println!("{} != {}", bs.len(), o); utils::flush(); utils::sleep(5000); }},
                      Err(_e) => { utils::flush(); }
                    }
                }
                let bs = glyph.ch.encode_utf8(&mut cb).as_bytes();
                match stdout().write(bs)  {
                    Ok(o) => { if o != bs.len() { utils::flush(); println!("{} != {}", bs.len(), o); utils::flush(); utils::sleep(5000); }},
                    Err(_e) => { utils::flush(); }
                }
            }
            col += 1;
            if col == self.term.cols() { col = 0; row += 1; } 
        }
        return self;
    } // Tbuff::dump

    pub fn flush (self :& Tbuff) -> &Self {
        utils::flush();
        return self;
    }

    pub fn new () -> Tbuff {
      let tb = Tbuff{
          buff : Vec::<Glyph>::new(),
          term : Term::new(),
          tick : 0
        };
       tb.term.terminalraw().cursoroff();
       return tb;
    }

    pub fn pp (self :& Tbuff) -> &Self {
        println!("{:?}", self.buff);
        return self;
    }

    pub fn done (self :& Tbuff) {
         self.term.done()
    }
} // impl Tbuff

/// Enhance Tbuff struct with more useful methods.
/// 
impl Tbuff {

    fn clear (self :&mut Tbuff,
            bg    :i32,
            fg    :i32,
            ch    :char) -> &mut Self {
        for y in 0..(self.rows()-0) {
            for x in 0..self.cols() {
                self.set(x, y, bg, fg, ch);
            }
        }
        return self;
    }

    fn draw_axis (self :&mut Tbuff) -> &mut Self{
        for y in 0..(self.rows()-0) {
            self.set(0, y, 0, 8, '|'); 
        }
    
        for x in 0..self.cols() {
            self.set(x, 0, 0, 8, '-');
        }
        self.set(0, 0 , 0, 8, '+');
        return self;
    }

    fn draw_background_sinies (
            self : &mut Tbuff,
            z    : i32) -> &mut Self {
        for y in 0..self.rows() {
            let h = 0.1 + 0.3 * ( 0.1 * (z as f32)).sin();
            let g = (6.28 / (24.0 + h) * (y as f32 * 95.0 + z as f32)).sin();
            for x in 0..self.cols() {
                let k = 0.3 + 0.3 * ( 0.1 * (z as f32)).sin();
                let j = (6.28 / (48.0 + k) * (y as f32 * 95.0 + x as f32+ z as f32)).sin();
                //let n = ((g + j) / 2.0 * 127.99 + 128.0) as i32;
                //let bg = (n/3) % 256;
                //self.set(x, y, bg*65536 + bg*256 + bg, bg*65536 + bg*256 + bg, ' '); 
                let n = ((g + j) / 2.0 * 11.99 + 12.0) as i32;
                let bg = (n/3) % 24 + 232;
                self.set(x, y, bg, bg, '.'); 
            }
        }
        return self;
    }

} // impl term::Tbuff


/// //////////// Test bf: /////////////////

fn fun_tbuff(term : &mut Term) {
  term.termsizeset(2,2); // Force Term to a 2x2
  let ref mut b = Tbuff::new(); // Init
  b.reset(0);                // Reset before each rendering
  b.set(0,0, 0, 7, 'x');
  b.set(1,0, 0, 7, 'y');
  b.set(2,0, 0, 7, 'z');
  b.pp();
}

pub fn main() {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
}
