#![allow(dead_code, unused_assignments, unused_imports, unused_variables, non_snake_case)]
use ::std::sync::{Arc, Mutex};
use ::std::thread::{spawn, JoinHandle};
use ::std::time::{SystemTime};
use ::std::ops::{Range};
use ::piston_window::*;

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

/// Copy arena to double buffer
fn arena_render (
    aa :&Arena,
    dd :&mut crate::dbuff::Dbuff
) {
    dd.tick(); // Tick the double buffer
    for y in (0..aa.len()).rev() {
        dd.put(&aa[y].lock().unwrap());
    }
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
    let mut a = 0; // Not set initially
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


fn genNewRows (
        aa      :&Arena,
        bb      :&Arena,
        range   :Range<usize>,
        w       :usize,
        h       :usize,
        threads :&mut Vec<JoinHandle<()>>,
        spins   :&Arc<Mutex<usize>>) {
    let aa = aa.clone();
    let bb = bb.clone();
    //let spin = spins.clone();
    let a = range.start;
    let b = range.end;
    threads.push(spawn( move || {
        //println!("genNewRows:: {:?} {:?}", a, b);
        for i in range { // Over the rows...
            loop { // Lock the four rows and generate new row or retry...
                let ral = aa[(i+h-1) % h].try_lock();
                if ral.is_err() {
                    //println!("a{:?} {} ", a..b-1, (i+h-1 % h) as i32);
                    //*spin.lock().unwrap() += 1;
                    continue
                }
                let rbl = aa[i].try_lock();
                if rbl.is_err() {
                    //println!("b{:?} {} ", a..b-1, i);
                    //*spin.lock().unwrap() += 1;
                    continue
                }
                let rcl = aa[(i+1) % h].try_lock();
                if rcl.is_err() {
                    //println!("c{:?} {} ", a..b-1, i+1%h);
                    //*spin.lock().unwrap() += 1;
                    continue
                }
                let bbl = bb[i].try_lock();
                if bbl.is_err() {
                    //println!("cbb {} ", i);
                    //*spin.lock().unwrap() += 1;
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
    }));
}


// Life ////////////////////////////////////////////////////////////////////////

pub struct Life {
    whs:    (usize, usize, usize),
    arenas: (Arena, Arena), // Nested mutable ADTs that can be passed to threads
    dbuffs: (crate::dbuff::Dbuff, crate::dbuff::Dbuff),// Piston is 2xbuffered so we need a dbuff for each
    pub tick: usize,
    state: bool,
    randomize: bool, // Randomizes field on next generation
    clear: bool,
    threads: usize,
    delta: usize,
}

impl Life { pub fn new (w: usize, h: usize) -> Life {
    Life {
        whs: (w, h, w*h),
        arenas: ( arena_new(w, h), arena_new(w, h) ),
        dbuffs: (
            crate::dbuff::Dbuff::new(w*h),
            crate::dbuff::Dbuff::new(w*h) ),
        tick: 0,
        state: false,
        randomize: false,
        clear: false,
        threads: 1,
        delta: 0
    }
} }

impl Life {
    pub fn tick (&mut self) -> &mut Self {
        self.tick += 1;
        self.state = !self.state;
        self
    }
    pub fn randomize (&mut self) -> &mut Self { self.randomize=true; self }
    pub fn clear (&mut self) -> &mut Self { self.clear=true; self }
    pub fn add_glider (&self, x: usize, y: usize) -> &Self {
        match self.state {
            false => draw_glider(&self.arenas.0, x, y),
            true  => draw_glider(&self.arenas.1, x, y)
        }
        self
    }
    pub fn render_dbuff (&mut self) -> &crate::dbuff::Dbuff {
        let aa = match self.state {
            false => &self.arenas.0,
            true  => &self.arenas.1
        };
        arena_render(aa, &mut self.dbuffs.0); // Copy to delta frame buffer
        self.tick();
        &self.dbuffs.0
    }
}

impl Life {
pub fn gen_next (&mut self) -> &mut Self {
    let (aa, bb) = // aa is the current arena (to read/dump), bb the next arena (to generate/overwrite)
        match self.state {
            false => (&self.arenas.0, &self.arenas.1),
            true  => (&self.arenas.1, &self.arenas.0)
        };

    let spins = Arc::new(Mutex::new(0)); // Keep track of mutex spins TODO

    if self.randomize {
        self.randomize = false;
        arena_randomize(bb, self.whs.0, self.whs.1);
    } else if self.clear {
        self.clear = false;
        arena_clear(bb, self.whs.0, self.whs.1);
    } else {
        // Spawn the threads to generate the next generation
        let mut threads :Vec<JoinHandle<()>> = vec!();
        let (w, h,_) = self.whs;
        for p in 0 .. self.threads {
            let range = h*p/self.threads .. h*(p+1)/self.threads;
            //print!("{:?} ", range);
            genNewRows(aa, bb, range, w, h, &mut threads, &spins);
        }
        //println!("");
        // Wait for threads to finish
        for t in 0..threads.len() { 
            //println!("Waiting on thread {}", t);
            threads.pop().unwrap().join().unwrap();
        }
    }
    self
}
}

impl Life {
pub fn render (
    &mut self,
    pwin:  &mut PistonWindow,
    event: Event,
    cellsize: f64
) -> &Self {
    let (aa, dd) = // aa is the current arena to render/read/dump
        match self.state {
            false => (&self.arenas.0, &mut self.dbuffs.0),
            true  => (&self.arenas.1, &mut self.dbuffs.1)
        };
    arena_render(aa, dd); // Copy to delta frame buffer
    let mut delta :usize = 0; // Count number of changes from last aren
    let (w, h, _) = self.whs;
    pwin.draw_2d(
        &event,
        |context:  piston_window::Context,
            graphics: &mut piston_window::G2d,
            _device:  &mut piston_window::GfxDevice
        | { dumpPiston(dd, &mut delta, w, h, context, graphics, cellsize); }
    );
    self.tick(); // Tick myself
    self.delta = delta;
    self
}
}

pub fn dumpPiston (
    this: &crate::dbuff::Dbuff,
    writes: &mut usize,
    width: usize,
    height: usize,
    context  :piston_window::Context,
    graphics :&mut G2d,
    cellsize: f64
) {
    let (ba, bb) = 
        match this.tick & 1 {
            0 => (&this.buffa, &this.buffb),
            _ => (&this.buffb, &this.buffa)
        };
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
            *writes += 1;
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
} // fn dumpPiston


////////////////////////////////////////////////////////////////////////////////

fn mainLife1 (w: usize, h: usize, cellsize: usize) {
    let mut life :Life = Life::new(w, h);
    let cellsize = 5;
    let winSize = (life.whs.0*cellsize, life.whs.1*cellsize);
    let mut epoch :SystemTime = SystemTime::now(); // For FPS calculation
    let mut frameCount = 0;
    let mut pwin: PistonWindow =
        WindowSettings::new("ASCIIRhOIDS", [winSize.0 as u32, winSize.1 as u32])
            //.exit_on_esc(true)
            .size(piston_window::Size{width: winSize.0 as f64, height: winSize.1 as f64})
            .decorated(true)
            .build()
            .unwrap();
    pwin.set_max_fps(120);

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

    //::util::sleep(1 * 1000);
    //println!("{:?}", event);

    while let Some(event) = pwin.next() {
        match event { // events.next(&mut pwin)
        Event::Loop(Loop::Idle(IdleArgs{dt})) => { },
        Event::Input(Input::Resize(ResizeArgs{window_size, draw_size}), _) => { },
        Event::Input(Input::Button(ButtonArgs{state:s, button:Button::Keyboard(k), scancode:_}), _) => {
            match k {
                Key::Space  => { life.randomize(); },
                Key::C      => { life.clear(); },
                Key::Q |
                Key::Escape => { pwin.set_should_close(true); },
                _ => ()
            }
        },
        Event::Loop(Loop::Render(args)) => {
            //println!("{:?}", args);
            //life.randomize();
            life.gen_next();
            life.render(&mut pwin, event, cellsize as f64);
            frameCount += 1;
        },
        _ => () } // match
        if 0 == life.tick % 50 { life.add_glider(10, 10); } // Draw a glider periodically
        if 50000 < life.tick { pwin.set_should_close(true); }
        if 1.0 < epoch.elapsed().unwrap().as_secs_f32() {
            println!("\x1b[0;35m threads:{} FPS:{:7.2} frame:{} deltas:{} spins:{}\x1b[40m  ",
                life.threads,
                frameCount as f32 / epoch.elapsed().unwrap().as_secs_f32(),
                life.tick,
                life.delta,
                0 //spins.lock().unwrap()
                );
            frameCount = 0;
            epoch = SystemTime::now();
        }
    } // while Some(event)
}

pub fn dumpLife (dbuff :& crate::dbuff::Dbuff, width: usize, idx: usize) {
    dbuff.get(idx).iter().enumerate().for_each( |(i, e)| {
        if 0 == i % width { println!("") }
        print!("{}", match *e { 0 => '.', 1 => '@', _ => '?'});
    } );
}

pub fn mainLife2 (w: usize, h: usize) {
    let mut life :Life = Life::new(w, h);
    let term = ::term::Term::new();
    term.terminalraw();

    loop {
        match term.getc().as_str() { "q" => { break; } _ => () }
        let dbuff = life.gen_next().render_dbuff();
        dumpLife(&dbuff, w, 0);
        println!("\x1b[H");
        if life.tick == 1000 { break; }
        if 0 == life.tick % 100 { life.add_glider(0,0); }
    }
    term.done();
}

pub fn main () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    mainLife2(140, 24);
    mainLife1(200, 100, 4);
}

// TODO: message passing pipeline
//   Verify a thread crashing with a lock and subsequent threads receiving
//   invalid locks can communicate the new state (machine).
/*
                println!("{:?}", event);
*/