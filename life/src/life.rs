//#![allow(dead_code, unused_assignments, unused_imports, unused_variables, non_snake_case)]
#![allow(non_snake_case)]
use ::std::sync::{Arc, Mutex};
use ::std::thread::{spawn, JoinHandle};
use ::std::ops::{Range};
use ::piston_window::*;

// Local libs
use ::util::{Watch};
use crate::dbuff::*;

/// Game of life

/// Random boolean integer 32 bit.  Returns 0 or 1 with probability 1/m.
pub fn rbi32(m: u32) -> i32 { ((::rand::random::<u32>() % m) == 0) as i32 }
pub fn rf32() -> f32 { ::rand::random::<f32>() }

pub fn pattern1 (x: i32, y: i32, s: i32) -> bool {
    (x*x + y*y) <= s && x.abs() == y.abs()
}
pub fn pattern2 (x: i32, y: i32, s: i32) -> bool {
    ((x*x + y*y) <= s) && (x == 0 || y == 0 || x.abs() == y.abs() )
}

// Arena ///////////////////////////////////////////////////////////////////////

type Arena = Arc<Vec<Mutex<Vec<i32>>>>;

fn arena_new (width: i32, height: i32, s: i32) -> Arena  {
    Arc::new(
        (0..height).map(
            |_h| Mutex::new(
                (0..width)
                .map( |_w| if s == 0 { 0 } else { rbi32(10) } )
                .collect::<Vec<_>>()))
        .collect::<Vec<Mutex<Vec<_>>>>())
}

fn draw_glider (aa :& Arena, x :usize, y :usize) {
    aa[y+0].lock().unwrap()[x+2] = 1;
    aa[y+1].lock().unwrap()[x+2] = 1;
    aa[y+2].lock().unwrap()[x+2] = 1;
    aa[y+2].lock().unwrap()[x+1] = 1;
    aa[y+1].lock().unwrap()[x+0] = 1;
}

fn arena_clear (bb :&Arena, w :usize, h :usize) {
    for y in 0..h {
        let mut row = bb[y].lock().unwrap();
        for x in 0..w { row[x] = 0; }
    }
}

fn arena_randomize (bb :&Arena, w :usize, h :usize, s: i32) {
    for y in 0..h {
        let mut row = bb[y].lock().unwrap();
        for x in 0..w {
             row[x] = { //rbi32(10);
                let xx = x as i32 - w as i32 /2;
                let yy = y as i32 - h as i32 /2;
                pattern2(xx, yy, s.pow(2)) as i32
             }
        }
    }
}

/// Update/mutate the next gen of life in row 'bb' given the current
/// row 'rb', the row above 'ra', and row below 'rc'.
fn genNewRow (
        w :usize,
        ra :& Vec<i32>,
        rb :& Vec<i32>,
        rc :& Vec<i32>,
        bb :&mut Vec<i32>) {
    // Sum of columns window
    let mut a; // Not set initially
    let mut b = ra[w-1] + rb[w-1] + rc[w-1]; // Last column of game field
    let mut c = ra[0]   + rb[0]   + rc[0];   // First column of game field
    let firstCol = c;
    let mut alive = rb[0];
    let mut nextAlive = 0;
    let mut k = 0; // Column index

    for j in 0..w { // Along the row
        // Shift colums left
        k = k + 1; // next column index
        a = b;
        b = c;
        c = if k==w { firstCol } else { nextAlive = rb[k]; ra[k] + rb[k] + rc[k] };
        //// Set the next generation cell given neighbor count:  3 or (2 and alive)
        bb[j] = ((( a + b + c << 1) as u32).wrapping_sub((alive+5)as u32) < 3) as i32;
        alive = nextAlive; // Consider next cell for next iteration
    }
}

fn spwnGenNextRows (
        aa      :&Arena,
        bb      :&Arena,
        range   :Range<usize>,
        w       :usize,
        h       :usize
) -> JoinHandle<()> {
    let aa = aa.clone();
    let bb = bb.clone();
    spawn( move || {
        //println!("spwnGenNextRows:: {:?} {:?}", range.start, range.end);
        //let (ra, rb) = (range.start, range.end);
        for i in range { // Over the rows...
            loop { // Lock the four rows and generate new row or retry...
                let ral = aa[(i+h-1) % h].try_lock();
                if ral.is_err() {
                    //println!("a{:?} {} ", ra..rb-1, (i+h-1 % h) as i32);
                    continue
                }
                let rbl = aa[i].try_lock();
                if rbl.is_err() {
                    //println!("b{:?} {} ", ra..rb-1, i);
                    continue
                }
                let rcl = aa[(i+1) % h].try_lock();
                if rcl.is_err() {
                    //println!("c{:?} {} ", ra..rb-1, i+1%h);
                    continue
                }
                let bbl = bb[i].try_lock();
                if bbl.is_err() {
                    //println!("cbb {} ", i);
                    continue
                }
                genNewRow(w,
                    & ral.unwrap(),
                    & rbl.unwrap(),
                    & rcl.unwrap(),
                    &mut bbl.unwrap());
                break;
            }
        }
    })
}


/// Life ////////////////////////////////////////////////////////////////////////
// Two arenas that write back and forth to each eacher  one row at a time:
//           ,------< one or more threads
//  Arena A  v  Arena B   
//  [     ] --> [     ]  DbuffB [    ]
//  [     ] --> [     ]
//  [     ] --> [     ]  DbuffA [    ]
//          \____________^  <--- single thread should happen first or interleave?
//
pub struct Life {
    whs:    (usize, usize, usize), // width height size
    arenas: (Arena, Arena), // Nested mutable ADTs that can be passed to threads
    pub dbuffs: (Arc<Mutex<Dbuff>>, Arc<Mutex<Dbuff>>),// Piston is 2xbuffered so we need a dbuff for each
    pub dbuff_en: bool,
    pub tick: usize,
    randomize: i32, // Randomizes field on next generation
    clear: bool,
    threads: usize, // How many threads to spanw for each generation computation
    pub threadvec :Vec<JoinHandle<()>>,
}

impl Life { pub fn new (w: usize, h: usize) -> Life {
    let mut this = Life {
        whs: (w, h, w*h),
        arenas: ( arena_new(w as i32, h as i32, 1), arena_new(w as i32, h as i32, 0) ),
        dbuffs: (
            Arc::new(Mutex::new(Dbuff::new(w*h, 0))),
            Arc::new(Mutex::new(Dbuff::new(w*h, 1)))),
        dbuff_en: true,
        tick: 0,
        randomize: 0,
        clear: false,
        threads: 6,
        threadvec: vec!()
    };
    this.arena_xfer_dbuff();
    this.tick = 1;
    this.arena_xfer_dbuff();
    this.tick = 0;
    this
} }

impl Life {
    pub fn tick (&mut self) -> &mut Self {
        self.tick += 1;
        self
    }
    pub fn randomize (&mut self, s: i32) -> &mut Self { self.randomize=s; self }
    pub fn clear (&mut self) -> &mut Self { self.clear=true; self }
    pub fn add_glider (&self, x: usize, y: usize) -> &Self {
        let aa = &if self.state() {&self.arenas.1} else {&self.arenas.0};
        draw_glider(aa, x, y);
        self
    }
    // Which arena generation state?
    //   False: Arena 0->1
    //   True:  Arena 1->0
    pub fn state (&self) -> bool { 1 == self.tick & 1 }

    pub fn gen_next (&mut self) {
        let (aa, bb) = // aa is the current arena (to read), bb the next arena (to overwrite)
            match self.state() {
                false => (&self.arenas.0, &self.arenas.1),
                true  => (&self.arenas.1, &self.arenas.0)
            };
        if self.randomize != 0 {
            arena_randomize(bb, self.whs.0, self.whs.1, self.randomize);
            self.randomize = 0;
        } else if self.clear {
            self.clear = false;
            arena_clear(bb, self.whs.0, self.whs.1);
        } else {
            let (w, h,_) = self.whs;
            for p in 0 .. self.threads {
                let range = h*p/self.threads .. h*(p+1)/self.threads;
                let t =
                spwnGenNextRows(aa, bb, range, w, h);
                self.threadvec.push(t);
            }
        }
        for _ in 0..self.threadvec.len() { self.threadvec.pop().unwrap().join().unwrap(); }
        self.arena_xfer_dbuff();
        self.tick();
    }

    pub fn cmdbuff (&self) -> &Arc<Mutex<Dbuff>> {
        match self.dbuff_en && self.state() {
            false => &self.dbuffs.0,
            true  => &self.dbuffs.1
        } 
    }
    
    pub fn arena_xfer_dbuff (&mut self) {
        let ab = match self.state() { // ab is the arena that was just generated
            false => &self.arenas.1,
            true  => &self.arenas.0
        };
        let mut dd = match !self.dbuff_en || self.state() {
            false => &self.dbuffs.1,
            true  => &self.dbuffs.0
        }.lock().unwrap(); 
        dd.tick(); // Tick double buffer, clears next active buffer
        // Copy each mutxed arena row to double-buffer
        for y in (0..ab.len()).rev() { // Reverse the indexing so as to only conflict with the nexst generation thread once in the vector.
            //dd.put(&ab[y].lock().unwrap());
            loop { match ab[y].try_lock() { // Spin lock acquire
                  Err(_) => { print!("L0"); ::util::sleep(1); continue } ,
                  Ok(t) => { dd.put(&t); break }
            } }
        }
    }

}

pub fn piston_draw_2d_callback (
    mdbuff: &Mutex<Dbuff>,
    whs: &(usize, usize, usize),
    deltas: &mut usize,
    graphics :&mut G2d,
) {
    let dbuff = mdbuff.lock().unwrap();
    let (ba, bb) = dbuff.buffs();
    let mut col :f32 = 0.0;
    let mut row :f32 = 0.0;
    //clear([0.0, 0.0, 0.0, 1.0], graphics);

    //let ds = DrawState { blend: None, stencil: None, scissor: None };
    let ds = DrawState { blend: Some(draw_state::Blend::Alpha), stencil: None, scissor: None };

    // Scale to NDC "Normalized device coordinate" (still need to translate -1,-1)
    let sx = 2.0 / whs.0 as f32;
    let sy = 2.0 / whs.1 as f32;
    let black = [0.0, 0.0, 0.0, 1.0];
    //let blue = [rf32(), rf32(), rf32(), 0.5];
    let blue = [0.0, 0.0, 1.0, 1.0];
    for i in 0..whs.2 {
        if ba[i] != bb[i] { // Compare buffer A with Buffer B for change in life state
            *deltas += 1;
            // Split the GoL square into two triangles
            let (fx, fy) = ( col * sx - 1.0, row * sy - 1.0 );
            let (gx, gy) = (fx + sx, fy + sy);
            let poly = [[fx,fy], [gx,fy], [gx,gy], [fx,fy], [gx,gy], [fx,gy]];
            if true || 0 != ba[i] {
                graphics.tri_list(&ds, &if 0!=ba[i]{blue}else{black}, |f| f(&poly));
            }
        }
        col += 1.0;
        if col == whs.0 as f32 { col = 0.0; row += 1.0; }
    }
    if false { // Slowly erase screen.  Disable recBlack plots for cool effect.
        graphics.tri_list(&ds, &[-1.0, -1.0, -1.0, 0.06],
             |f| f(&maketriangle()[..]));
    }
    //rectangle( [ 1.0, 0.0, 0.0, 1.0 ], [ 50.0, 50.0, 50.0, 50.0 ], context.transform, graphics);
} // fn piston_draw_2d_callback

fn maketriangle () -> [[f32;2];6] {
    [ [-1.0,-1.0], [1.0,-1.0], [1.0,1.0],
      [-1.0,-1.0], [1.0,1.0],  [-1.0,1.0] ]
}

pub fn piston_render (
    mdbuff: &Mutex<Dbuff>,
    whs: &(usize, usize, usize),
    pwin: &mut PistonWindow,
    event: Event,
) -> usize {
    let mut deltas :usize = 0; // Count number of changes from last aren
    pwin.draw_2d( &event,
        | _context:  piston_window::Context,
          graphics: &mut piston_window::G2d,
          _device:  &mut piston_window::GfxDevice
        | {
        piston_draw_2d_callback(mdbuff, whs, &mut deltas, graphics);
    });
    deltas
}


////////////////////////////////////////////////////////////////////////////////

fn main_life_2d (w: usize, h: usize, cellsize: usize) -> bool {
    let mut life :Life = Life::new(w, h);
    let mut deltas :usize = 0;
    let winSize = (life.whs.0*cellsize, life.whs.1*cellsize);
    let mut watch = Watch::new();
    let mut pwin: PistonWindow =
        WindowSettings::new("ASCIIRhOIDS", [winSize.0 as u32, winSize.1 as u32])
            //.exit_on_esc(true)
            .size(piston_window::Size{width: winSize.0 as f64, height: winSize.1 as f64})
            .decorated(true)
            .build()
            .unwrap();
    let mut s = 1;
    pwin.set_max_fps(50);

        /* Ghetto dump arena
        for y in 0..24 {
            println!("");
            let row = aa[y].lock().unwrap();
            for x in 0..80 {
                    print!("{}", 1-row[x]);
            }
        }
        print!("\x1b[H");
        */


    //let mut d = ::util::delta();
    // First time this is called, it will be overwriting Arena B with Arena A's next gen and subsequenty copy the arena to dbuff B.
    // IN the mean time, dbuff A can be read and rednered.
    //life.tick();
    // Tick should represent which Arena is readable.  tick==0 implies Arena A is readable
    // After first gen_next, tick should be incremented to indicated Arena B should be read
    while let Some(event) = pwin.next() {
        match event { // events.next(&mut pwin)
        Event::Loop(Loop::Idle(IdleArgs{dt:_})) => { },
        Event::Input(Input::Resize(ResizeArgs{window_size:_, draw_size:_}), _) => { },
        Event::Input(Input::Button(ButtonArgs{state:st, button:Button::Keyboard(k), scancode:_}), _) => {
            if st == ButtonState::Press {
                match k {
                    Key::Space  => { s+=1; life.randomize(s);},
                    Key::C      => { life.clear(); },
                    Key::Q |
                    Key::Escape => { pwin.set_should_close(true); },
                    _ => ()
                }
            }
        },
        Event::Loop(Loop::Render(_args)) => {
            //println!("{:?}", d());
            let mdbuff = life.cmdbuff().clone();
            let whs = life.whs;
            life.gen_next(); // Start next gen while current gen is rendered
            deltas = piston_render(&mdbuff, &whs, &mut pwin, event);
            //d();
            //if 1==watch.tick { life.add_glider(0, 0); }
            watch.tick();
        },
        _ => ()
        } // match

        if let Some(w) = watch.mark(2.0) {
            //life.add_glider(0, 0);
            println!("\x1b[0;35m{} [{:.2}]\tgens:{} ∆lifes:{} spins:{} s={}\x1b[40m",
                life.threads, w.fps, life.tick, deltas, 0, s);
        }
        if 50000 < life.tick { pwin.set_should_close(true); }
    } // while Some(event)
    true
}

fn main_life_ascii (w: usize, h: usize, loopcount: usize) {
    let mut watch = util::Watch::new();
    let mut life :Life = Life::new(w, h);
    life.dbuff_en = false;
    let term = ::term::Term::new();
    term.terminalraw();
    loop {
        if 0 == life.tick % 100 { life.add_glider(0,0); }
        let mdbuff = life.cmdbuff().clone();
        life.gen_next();
        watch.tick().mark(1.0);
        print!("\x1b[H Game of Life -- ASCII edition {}", watch.fps);
        // Dump to terminal
        /*
        for _ in 0..life.threadvec.len() {  // Wait for threads
            print!(". ");
            life.threadvec.pop().unwrap().join().unwrap();
        }
        */
        mdbuff.lock().unwrap().buff().iter().enumerate().for_each( |(i, e)| {
            if 0 == i % w { println!("") }
            print!("{}", if *e == 0 { '.' } else { '@'});
        } );
        match &term.getc()[..] {
          "q" => break,
          "c" => { life.clear(); }
          " " => { life.randomize(8); }
          _ => ()
        }
        if 0 < loopcount && life.tick == loopcount { break; }
    }
    term.done();
}

pub fn main () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    if !false  { main_life_2d(300, 300, 3); }
    if !true { main_life_ascii(140, 24, 10000); }
    //crate::dbuff::main();
}

// TODO: message passing pipeline
//   Verify a thread crashing with a lock and subsequent threads receiving
//   invalid locks can communicate the new state (machine)/
/*
                println!("{:?}", event);
*/