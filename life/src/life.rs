//#![allow(dead_code, unused_assignments, unused_imports, unused_variables, non_snake_case)]
#![allow(non_snake_case)]
use ::std::sync::{Arc, Mutex};
use ::std::thread::{spawn, JoinHandle};
//use ::std::time::{SystemTime};
use ::std::ops::{Range};
use ::piston_window::*;
use ::util::{Watch};

/// Game of life

/// Random boolean integer 32 bit.  Returns 0 or 1 with probability 1/m.
pub fn rbi32(m: u32) -> i32 { ((::rand::random::<u32>() % m) == 0) as i32 }

// Arena ///////////////////////////////////////////////////////////////////////

type Arena = Arc<Vec<Mutex<Vec<i32>>>>;

fn arena_new (w: usize, h: usize) -> Arena  {
    Arc::new(
        (0..h).map(
            |_| Mutex::new(
                (0..w).map(|_| rbi32(10) )
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

fn arena_randomize (bb :&Arena, w :usize, h :usize) {
    for y in 0..h {
        let mut row = bb[y].lock().unwrap();
        for x in 0..w { row[x] = rbi32(10); }
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
        k = k + 1; // next column index
        // Shift colums left
        a = b;
        b = c;
        c = if k==w { firstCol } else { nextAlive = rb[k]; ra[k] + rb[k] + rc[k] };
        let lives = a + b + c - alive; // Window lives count minus the current live value
        // Set the next generation cell given neighbor count:  3 or 2 and alive
        bb[j] = (3 == lives || (2 == lives && 1 == alive)) as i32; // Next gen exists if 3 neighbors or born if 2 neighbors.
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


// Life ////////////////////////////////////////////////////////////////////////

pub struct Life {
    whs:    (usize, usize, usize), // width height size
    arenas: (Arena, Arena), // Nested mutable ADTs that can be passed to threads
    pub dbuffs: (crate::dbuff::Dbuff, crate::dbuff::Dbuff),// Piston is 2xbuffered so we need a dbuff for each
    pub dbuff_en: bool,
    pub tick: usize,
    randomize: bool, // Randomizes field on next generation
    clear: bool,
    threads: usize, // How many threads to spanw for each generation computation
    pub threadvec :Vec<JoinHandle<()>>,
}

impl Life {
pub fn new (w: usize, h: usize) -> Life {
    Life {
        whs: (w, h, w*h),
        arenas: ( arena_new(w, h), arena_new(w, h) ),
        dbuffs: (
            crate::dbuff::Dbuff::new(w*h),
            crate::dbuff::Dbuff::new(w*h) ),
        dbuff_en: true,
        tick: 0,
        randomize: false,
        clear: false,
        threads: 1,
        threadvec: vec!()
    }
} }

impl Life {
    pub fn tick (&mut self) -> &mut Self {
        self.tick += 1;
        self
    }
    pub fn randomize (&mut self) -> &mut Self { self.randomize=true; self }
    pub fn clear (&mut self) -> &mut Self { self.clear=true; self }
    pub fn add_glider (&self, x: usize, y: usize) -> &Self {
        match self.state() {
            false => draw_glider(&self.arenas.0, x, y),
            true  => draw_glider(&self.arenas.1, x, y)
        }
        self
    }
    // Which arena generation state?
    //   False: Arena 0->1
    //   True:  Arena 1->0
    pub fn state (&self) -> bool { 1 == self.tick & 1 }

    pub fn gen_next (&mut self) -> &mut Self {
        let (aa, bb) = // aa is the current arena (to read), bb the next arena (to overwrite)
            match self.state() {
                false => (&self.arenas.0, &self.arenas.1),
                true  => (&self.arenas.1, &self.arenas.0)
            };
        if self.randomize {
            self.randomize = false;
            arena_randomize(bb, self.whs.0, self.whs.1);
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
        self
    }

    pub fn dbuff (&mut self) -> &mut crate::dbuff::Dbuff {
        match self.dbuff_en && self.state() {
            false => &mut self.dbuffs.0,
            true  => &mut self.dbuffs.1
        } 
    }
    
    pub fn arena_xfer_dbuff (&mut self) -> &mut Self {
        let aa = match self.state() { // aa is the current arena to read/render from
            false => &self.arenas.0,
            true  => &self.arenas.1
        };
        let dd = match self.dbuff_en && self.state() {
            false => &mut self.dbuffs.0,
            true  => &mut self.dbuffs.1
        }; 
        dd.tick(); // Tick double buffer, clears next active buffer
        // Copy arena to double buffer
        for y in (0..aa.len()).rev() { // Reverse the indexing so as to only conflict with the nexst generation thread once in the vector.
            //dd.put(&aa[y].lock().unwrap());
            loop { match aa[y].try_lock() { // Spin lock acquire
                  Err(_) => continue,
                  Ok(t) => { dd.put(&t); break }
            } }
        }
        self
    }

}

pub fn piston_draw_2d_callback (
    dbuff: &crate::dbuff::Dbuff,
    deltas: &mut usize,
    width: usize,
    height: usize,
    context  :piston_window::Context,
    graphics :&mut G2d,
    cellsize: f64
) {
    let (ba, bb) = dbuff.buffs();
    let mut col :usize = 0;
    let mut row :usize = 0;
    //clear([0.0, 0.0, 0.0, 1.0], graphics);

    let recBlue  = Rectangle{
        color: [0.0, 0.0, 1.0, 1.0],
        shape: rectangle::Shape::Square,
        border: None //Some(rectangle::Border{ color:[0.0, 1.0, 0.0, 1.0], radius:0.3 })
    };
    let recBlack = Rectangle{
        color: [0.0, 0.0, 0.0, 1.0],
        shape: rectangle::Shape::Square,
        border: None //Some(rectangle::Border{ color:[0.0, 0.0, 0.0, 1.0], radius:0.3 })
    };

    let drawState = DrawState {
        blend: None, //Some(draw_state::Blend::Alpha),
        stencil: None,
        scissor: None};
    let mut poly = [0.0, 0.0, 0.0, 0.0];
    let xform = context.transform;
    for i in 0..width*height {
        if ba[i] != bb[i] {
            *deltas += 1;
            poly[0] = col as f64 * cellsize;
            poly[1] = row as f64 * cellsize;
            poly[2] = cellsize;
            poly[3] = cellsize;
            if 0 != ba[i] {
                recBlue.draw_tri(poly, &drawState, xform, graphics);
            }else {
                recBlack.draw_tri(poly, &drawState, xform, graphics);
            }
        }
        col += 1;
        if col == width { col = 0; row += 1; }
    }
    if false {
    Rectangle{
        color: [0.0, 0.0, 0.0, 0.02],
        shape: rectangle::Shape::Square,
        border: None
    }.draw_tri(
        [-1.0, -1.0, 2.0, 2.0],
        &DrawState {
            blend: Some(draw_state::Blend::Alpha),
            stencil: None,
            scissor: None},
        [[1.0,0.0,0.0],
         [0.0,1.0,0.0]],
        graphics);
    }
    //rectangle( [ 1.0, 0.0, 0.0, 1.0 ], [ 50.0, 50.0, 50.0, 50.0 ], context.transform, graphics);
} // fn piston_draw_2d_callback

pub fn piston_render (
    life: &mut Life,
    //dbuff: &crate::dbuff::Dbuff,
    pwin: &mut PistonWindow,
    event: Event,
    cellsize: f64
) -> usize {
    let w = life.whs.0;
    let h = life.whs.1;
    let dbuff = life.dbuff();
    let mut deltas :usize = 0; // Count number of changes from last aren
    pwin.draw_2d(
        &event,
        |context:  piston_window::Context,
            graphics: &mut piston_window::G2d,
            _device:  &mut piston_window::GfxDevice
        | { piston_draw_2d_callback(dbuff, &mut deltas, w, h, context, graphics, cellsize); }
    );
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
    pwin.set_max_fps(1000);

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
    life.gen_next();
    while let Some(event) = pwin.next() {
        match event { // events.next(&mut pwin)
        Event::Loop(Loop::Idle(IdleArgs{dt:_})) => { },
        Event::Input(Input::Resize(ResizeArgs{window_size:_, draw_size:_}), _) => { },
        Event::Input(Input::Button(ButtonArgs{state:_, button:Button::Keyboard(k), scancode:_}), _) => {
            match k {
                Key::Space  => { life.randomize(); },
                Key::C      => { life.clear(); },
                Key::Q |
                Key::Escape => { pwin.set_should_close(true); },
                _ => ()
            }
        },
        Event::Loop(Loop::Render(_args)) => {
            life.arena_xfer_dbuff();
            // Wait for threads to finish
            print!("{} ", life.threadvec.len());
            for _ in 0..life.threadvec.len() { 
                print!(". ");
                life.threadvec.pop().unwrap().join().unwrap();
            }
            //println!("{:?}", d());
            deltas = piston_render(&mut life, &mut pwin, event, cellsize as f64);
            //d();
            life.tick();
            watch.tick();
            life.gen_next();
        },
        _ => () } // match

        if let Some(w) = watch.mark(1.0) {
            life.add_glider(0, 0);
            println!("\x1b[0;35m |{}| [{:.2}] frame:{} deltas:{} spins:{}\x1b[40m  ",
                life.threads,
                w.fps,
                life.tick,
                deltas,
                0 //spins.lock().unwrap()
                );
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
        life.gen_next();
        life.arena_xfer_dbuff();
        watch.tick().mark(1.0);
        print!("\x1b[H Game of Life -- ASCII edition {}", watch.fps);
        // Dump to terminal
        /*
        for _ in 0..life.threadvec.len() {  // Wait for threads
            print!(". ");
            life.threadvec.pop().unwrap().join().unwrap();
        }
        */
        life.dbuff().buff().iter().enumerate().for_each( |(i, e)| {
            if 0 == i % w { println!("") }
            print!("{}", if *e == 0 { '.' } else { '@'});
        } );
        life.tick();
        match &term.getc()[..] {
          "q" => break,
          "c" => { life.clear(); }
          _ => ()
        }
        if 0 < loopcount && life.tick == loopcount { break; }
    }
    term.done();
}

pub fn main () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    if true { main_life_2d(140, 24, 10); }
    if false  { main_life_ascii(140, 24, 10000); }
    //crate::dbuff::main();
}

// TODO: message passing pipeline
//   Verify a thread crashing with a lock and subsequent threads receiving
//   invalid locks can communicate the new state (machine).
/*
                println!("{:?}", event);
*/