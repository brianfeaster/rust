//pub mod term;

use crate::util;
use std::fmt;
use std::io::{stdin, stdout, Read, Write};
use std::sync::mpsc::{channel, Receiver};
use std::thread;


/// Implementation for "Pretty Printing" an object.
pub trait PrettyPrint {
    fn pp(&self);
}

/// Terminal Abstaction
pub struct Term {
    cols: i32,
    rows: i32,
    count: i32,
    original_termios: libc::termios,
    original_fcntl: libc::c_int,
    key : Receiver<String>,
}

impl Term {

    pub fn termsizeset(&mut self, cols: i32, rows: i32)  -> &Self {
        self.cols  = cols;
        self.rows  = rows;
        self.count = self.cols * self.rows as i32;
        return self;
    }

    pub fn termsize(&mut self) -> &Self {
        let winsize = libc::winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        unsafe {
            libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &winsize);
        }
        return self.termsizeset(winsize.ws_col as i32, winsize.ws_row as i32);
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
            util::flush();
            libc::fcntl(0, libc::F_SETFL, &self.original_fcntl | libc::O_NONBLOCK);
        }
        return self;
    }

    pub fn termblock(&self) -> &Self {
        unsafe {
            util::flush();
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

    pub fn cols (self: &Term) -> i32 { return self.cols; }
    pub fn rows (self: &Term) -> i32 { return self.rows; }
    pub fn count (self: &Term) -> i32 { return self.count; }

    /// Blocking read from terminal.  Unbuffered if terminalraw() called.
    fn getc_actual () -> String {
        let mut buffer = [0; 256];
        let mut count = 0;
        match stdin().read(&mut buffer) {
            Ok(result) => { count = result; },
            Err(_msg) => { }
        }
        return std::str::from_utf8(&buffer[0..count]).expect("utf8 issue").to_string();
    }

    pub fn getc (&self) -> String {
        let mut ss : String = String::new();
        loop {
            match self.key.try_recv() {
                Ok(s) => {ss = ss + &s; },
                Err(e) =>  { break }
            };
        }
        return ss;
    }

    pub fn new () -> Self {
        let (tx, rx) = channel();
        let mut term = Term{
            cols: 0,
            rows: 0,
            count: 0,
            original_termios: libc::termios {
                c_iflag: 0,
                c_oflag: 0,
                c_cflag: 0,
                c_lflag: 0,
                c_cc: [0; 20],
                c_ispeed: 0,
                c_ospeed: 0 },
            original_fcntl: 0,
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
                    eprintln!("crate::term::Term::new - thread Term::getc_actual is empty...looping...");
                    util::sleep(50);
                } else {
                  //println!("'TERM{} {}'", s, s.len());
                  //if s.as_bytes()[0] as char == 'Q' { break; }
                  tx.send(s).expect("Unable to send on channel");
                }
            }
        });
        return term;
    }

    pub fn done (&self) {
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &self.original_termios);
            //self.termblock();
        }
        print!("\x1b[0m"); // Reset color
        print!("\x1b[?25h"); // Enable Cursor
        //print!("\x1b[{}H", self.rows); // Cursor to screen bottom
    }

}

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
#[derive(Debug, Clone)]
pub struct Glyph {
    pub bg: i32,
    pub fg: i32,
    pub ch: char
}

pub struct Tbuff {
    buff: Vec<Glyph>,
    term: Term
}

const GLYPH_EMPTY :Glyph = Glyph{bg: 0, fg: 0, ch:' '};

impl Tbuff {

    pub fn getc (&self) -> String{
       self.term.getc()
    }

    pub fn cols (&self) -> i32 { self.term.cols() }
    pub fn rows (&self) -> i32 { self.term.rows() }
    pub fn count (&self) -> i32 { self.term.count() }

    pub fn reset (&mut self) -> &Self {
        self.term.termsize();
        self.buff.resize(self.term.count() as usize, GLYPH_EMPTY);
        return self;
    }

    pub fn set (&mut self, x:i32, y:i32, bg:i32, fg:i32, ch:char){
        use std::fmt::Write;
        //x += self.term.cols()/2;
        //y += self.term.rows()/2;
        let idx = ( self.cols() * y.rem_euclid(self.rows())
                    + x.rem_euclid(self.cols())) as usize;
        //self.buff[idx].clear();
        self.buff[idx].bg = bg;
        self.buff[idx].fg = fg;
        self.buff[idx].ch = ch;
    }

    pub fn line (&mut self, vs1 :&[f32], vs2 :&[f32], ch :char, color: i32) {
        let mut x = vs1[0] as i32;
        let mut y = vs1[1] as i32;
        for [xinc, yinc] in util::Walk::new(vs1, vs2) {
            x += xinc;
            y += yinc;
           self.set(x, y, 0, color, ch);
        }
    }
    
    pub fn dump (
        self :&Tbuff) -> &Self {
        let mut lbg: i32 = -1;
        let mut lfg: i32 = -1;
        let mut cb :[u8;4] = [0,0,0,0];
        match stdout().write("\x1b[H\x1b[0m".as_bytes()) {
            Ok(o) => {  },
            Err(e) => { util::flush(); }
        }
        for glyph in &self.buff {
            if lfg != glyph.fg {
                lfg = glyph.fg;
                let bs = if lfg < 8 {
                    format!("\x1b[3{}m", lfg)
                } else {
                    format!("\x1b[38;5;{}m", lfg)
                };
                match stdout().write(bs.as_bytes()) {
                  Ok(o) => { if o != bs.len() { util::flush(); println!("{} != {}", bs.len(), o); util::flush(); util::sleep(5000); }},
                  Err(e) => { util::flush(); }
                }
            }
            if lbg != glyph.bg {
                lbg = glyph.bg;
                let bs = if lfg < 8 {
                    format!("\x1b[4{}m", lbg)
                } else {
                    format!("\x1b[48;5;{}m", lbg)
                };
                match stdout().write(bs.as_bytes()) {
                  Ok(o) => { if o != bs.len() { util::flush(); println!("{} != {}", bs.len(), o); util::flush(); util::sleep(5000); }},
                  Err(e) => { util::flush(); }
                }
            }
            let bs = glyph.ch.encode_utf8(&mut cb).as_bytes();
            match stdout().write(bs)  {
                Ok(o) => { if o != bs.len() { util::flush(); println!("{} != {}", bs.len(), o); util::flush(); util::sleep(5000); }},
                Err(e) => { util::flush(); }
            }
        }
        return self;
    } // Tbuff::dump

    pub fn flush (self :&Tbuff) -> &Self {
        util::flush();
        return self;
    }

    pub fn new () -> Tbuff {
      let tb = Tbuff{
          buff : Vec::<Glyph>::new(),
          term : Term::new()
        };
       tb.term.terminalraw().cursoroff();
       return tb;
    }

    pub fn pp (&self) {
        println!("{:?}", self.buff);
    }

    pub fn done (&self) {
         self.term.done()
    }
} // impl Tbuff

/// //////////// Test bf: /////////////////
/// 
fn fun_tbuff(term : &mut Term) {
  term.termsizeset(2,2); // Force Term to a 2x2
  let ref mut b = Tbuff::new(); // Init
  b.reset();                // Reset before each rendering
  b.set(0,0, 0, 7, 'x');
  b.set(1,0, 0, 7, 'y');
  b.set(2,0, 0, 7, 'z');
  b.pp();
}
