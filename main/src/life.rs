use ::std::sync::{Arc, Mutex};
use ::std::thread::{spawn, JoinHandle};
use ::std::time::{SystemTime};
use ::std::ops::{Range};
use ::piston_window::*;

////////////////////////////////////////////////////////////////////////////////
/// TODO: message passing pipeline
///   Verify a thread crashing with a lock and subsequent threads receiving
///   invalid locks can communicate the new state (machine).
///

////////////////////////////////////////////////////////////////////////////////
/// Game of life state machine.  Let's try and make this concept happen.
///
#[derive(Debug)]
struct State {
    power :bool,
    randomize :bool,
    spin: usize,
    tick: usize,
    key: String,
    width: usize,
    height: usize,
    threads: usize
}

impl State {
    pub fn next (self :&mut State,
                 tb :& crate::term::Tbuff) -> &State {
        self.tick += 1;
        //self.randomize = false;
        self.key = tb.getc();
        match self.key.as_str() {
            "q" => self.power = false,
            " " => self.randomize = true,
            _ => ()
        }
        self
    }
    pub fn powered (self :& State) -> bool { self.power }

    pub fn randomize (self :&mut State) -> bool {
        let r = self.randomize;
        self.randomize=false;
        r
    }

    pub fn new () -> State {
        State {
            power :true,
            tick :0,
            randomize :true,
            key  :"".to_string(),
            spin: 0,
            width: 0,
            height: 0,
            threads: 1
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
/// Game of life
///

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
        h       :usize,
        w       :usize,
        threads :&mut Vec<JoinHandle<()>>,
        spins   :&Arc<Mutex<usize>>) {
    let aa = aa.clone();
    let bb = bb.clone();
    //let spin = spins.clone();
    let a = range.start;
    let b = range.end;
    threads.push(spawn( move || {
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

type Arena = Arc<Vec<Mutex<Vec<i32>>>>;

fn arena_render (
    aa :&Arena,
    tt :&mut crate::term::Tbuff
) {
    for y in (0..tt.rows()).rev() {
        let row = aa[y].lock().unwrap();
        for x in 0..tt.cols() {
            if 0 != row[x] {
                tt.set(x, y, 4, 12, '◼') // ▪ ◾ ◼ ■ █
            } else {
               //tt.set(x, y, 3, 0, ' ')
            }
       }
    }
}

fn arena_randomize (bb :&Arena, height :usize, width :usize) {
    for y in 0..height {
        let mut row = bb[y].lock().unwrap();
        for x in 0..width {
            row[x] = if 0 == crate::ri32(10) { 1 } else { 0 };
       }
    }
}

const ARENA_WIDTH :usize = 200; // 1430;
const ARENA_HEIGHT :usize = 100; // 423;

fn arena_new () -> Arena  {
    let mut arena = Arc::new(vec![]);
    let rows = Arc::get_mut(&mut arena).unwrap();
    for y in 0..ARENA_HEIGHT {
        rows.push(
            Mutex::new(
                (0 .. ARENA_WIDTH).map(|_| if 0 == crate::ri32(10) { 1 } else { 0 })
                    .collect::<Vec<i32>>()));
    }
    return arena;
}

fn draw_glider (aa :& Arena, x :usize, y :usize) {
        aa[10].lock().unwrap()[10] = 1;
        aa[11].lock().unwrap()[10] = 1;
        aa[12].lock().unwrap()[10] = 1;
        aa[12].lock().unwrap()[9] = 1;
        aa[11].lock().unwrap()[8] = 1;
}

fn life () {
    // Coneable objects that can be passed to threads
    let arena :Arena = arena_new();
    let arenb :Arena = arena_new();
    let mut ta = crate::term::Tbuff::new();
    let mut tb = crate::term::Tbuff::new();
    let mut state :State = State::new();
    let epoch :SystemTime = SystemTime::now(); // For FPS calculation
    let mut threads :Vec<JoinHandle<()>> = vec!();
    let mut pwin :PistonWindow =
        WindowSettings::new("ASCIIRhOIDS", [256 as u32, 256 as u32])
            .exit_on_esc(true)
            .decorated(true)
            .build()
            .unwrap();
    let spins = Arc::new(Mutex::new(0));

    state.width = ARENA_WIDTH;
    state.height = ARENA_HEIGHT;

    while state.powered() && state.tick < 50000 { // Loop until keypress 'q'

        //util::sleep(200);
        
        let (aa, bb, tt) = // aa is the current arena (to read/dump), bb the next arena (to generate/overwrite)
            match 0 == state.tick & 1 {
                true  => (&arena, &arenb, &mut ta),
                false => (&arenb, &arena, &mut tb)
            };

        let h = state.height;
        let w = state.width;
        tt.reset_size_and_tick(w, h);

        // Draw a glider periodically
        if 0 == state.tick % 14 { draw_glider(aa, 5, 5); }

        for _ in 0..threads.len() { threads.pop().unwrap().join().unwrap(); }

        // Either randomize the visible field or compute next generation
        if state.randomize() {
            arena_randomize(bb, h, w);
        } else {
            match state.threads {
            1 => {
               genNewRows(aa, bb, 0..h, h, w, &mut threads, &spins); // 240
            },

            2 => {
               genNewRows(aa, bb, 0   .. h/2, h, w, &mut threads, &spins); // 470
               genNewRows(aa, bb, h/2 .. h,   h, w, &mut threads, &spins);
            },

            3 => {
                genNewRows(aa, bb, 0     .. h/3,   h, w, &mut threads, &spins); // 620
                genNewRows(aa, bb, h/3   .. h*2/3, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*2/3 .. h,     h, w, &mut threads, &spins);
            },

            4 => {
                genNewRows(aa, bb, h*0/4 .. h*1/4, h, w, &mut threads, &spins); // 710
                genNewRows(aa, bb, h*1/4 .. h*2/4, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*2/4 .. h*3/4, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*3/4 .. h*4/4, h, w, &mut threads, &spins);
            },

            5 => {
                genNewRows(aa, bb, h*0/5 .. h*1/5, h, w, &mut threads, &spins); // 720
                genNewRows(aa, bb, h*1/5 .. h*2/5, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*2/5 .. h*3/5, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*3/5 .. h*4/5, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*4/5 .. h*5/5, h, w, &mut threads, &spins);
            },

            6 => {
                genNewRows(aa, bb, h*0/6 .. h*1/6, h, w, &mut threads, &spins); // 800
                genNewRows(aa, bb, h*1/6 .. h*2/6, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*2/6 .. h*3/6, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*3/6 .. h*4/6, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*4/6 .. h*5/6, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*5/6 .. h*6/6, h, w, &mut threads, &spins);
            },

            7 => {
                genNewRows(aa, bb, h*0/7 .. h*1/7, h, w, &mut threads, &spins); // 820
                genNewRows(aa, bb, h*1/7 .. h*2/7, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*2/7 .. h*3/7, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*3/7 .. h*4/7, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*4/7 .. h*5/7, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*5/7 .. h*6/7, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*6/7 .. h*7/7, h, w, &mut threads, &spins);
            },

            8 => {
                genNewRows(aa, bb, h*0/8 .. h*1/8, h, w, &mut threads, &spins); // 830
                genNewRows(aa, bb, h*1/8 .. h*2/8, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*2/8 .. h*3/8, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*3/8 .. h*4/8, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*4/8 .. h*5/8, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*5/8 .. h*6/8, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*6/8 .. h*7/8, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*7/8 .. h*8/8, h, w, &mut threads, &spins);
            },

            9 => {
                genNewRows(aa, bb, h*0/9 .. h*1/9, h, w, &mut threads, &spins); // 700
                genNewRows(aa, bb, h*1/9 .. h*2/9, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*2/9 .. h*3/9, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*3/9 .. h*4/9, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*4/9 .. h*5/9, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*5/9 .. h*6/9, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*6/9 .. h*7/9, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*7/9 .. h*8/9, h, w, &mut threads, &spins);
                genNewRows(aa, bb, h*8/9 .. h*9/9, h, w, &mut threads, &spins);
            },
            _ => ()
            } // match
        }

        for _ in 0..threads.len() { threads.pop().unwrap().join().unwrap(); }

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

        // Draw the arena.
        let render =
            0 == state.tick % 1;

        if render {
            arena_render(&aa, tt);
            while let Some(event) = pwin.next() {
                //println!("{:?}", event);
                //if event.idle_args() != None { break; }
                if event.text_args() != None {
                    if event.text_args().unwrap() == "q" { state.power = false; }
                    if event.text_args().unwrap() == " " { state.randomize = true; }
                } else if event.render_args() != None {
                    if render {
                        pwin.draw_2d(
                            &event,
                            |context: piston_window::Context,
                             graphics,
                             _device| {
                                tt.dumpPiston(context, graphics);
                            }
                        );
                    }
                    break;
                }
            } // while event
        } // render

        //tt.dump().flush();
        if 0 == state.tick % 100  {
            print!("\n\x1b[0;35m t:{} FPS:{:7.2} F:{} S:{}\x1b[40m  ",
                state.threads,
                1000.0 * state.tick as f32 / epoch.elapsed().unwrap().as_millis() as f32,
                state.tick,
                spins.lock().unwrap());
            util::flush();
        }

        state.next(&tt); // Needs to be mutable
    }
    ta.done();
    println!("{}", epoch.elapsed().unwrap().as_millis());
}

pub fn testA () {
    let mut a = vec!(0,0,0,0,0);
    let mut b = vec!(0,1,1,1,0);
    let mut c = vec!(0,0,0,1,0);
    let mut d = vec!(0,0,1,0,0);
    let mut e = vec!(0,0,0,0,0);

    let mut A = vec!(0,0,0,0,0);
    let mut B = vec!(0,0,0,0,0);
    let mut C = vec!(0,0,0,0,0);
    let mut D = vec!(0,0,0,0,0);
    let mut E = vec!(0,0,0,0,0);

    loop {
    genNewRow(5, &e, &a, &b, &mut A);
    genNewRow(5, &a, &b, &c, &mut B);
    genNewRow(5, &b, &c, &d, &mut C);
    genNewRow(5, &c, &d, &e, &mut D);
    genNewRow(5, &d, &e, &a, &mut E);
    println!("{:?}", &A);
    println!("{:?}", &B);
    println!("{:?}", &C);
    println!("{:?}", &D);
    println!("{:?}\x1b[H", &E);
    util::sleep(100);
    genNewRow(5, &E, &A, &B, &mut a);
    genNewRow(5, &A, &B, &C, &mut b);
    genNewRow(5, &B, &C, &D, &mut c);
    genNewRow(5, &C, &D, &E, &mut d);
    genNewRow(5, &D, &E, &A, &mut e);
    println!("{:?}", &a);
    println!("{:?}", &b);
    println!("{:?}", &c);
    println!("{:?}", &d);
    println!("{:?}\x1b[H", &e);
    util::sleep(100);
    }
}

pub fn main () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    self::life();
}

/*

NOTES
* Glider reflectors

////Spin until lock acquired
//let mut spin = 0; // keep track of busy waiting/spinning
//let bb = loop { match Arc::get_mut(&mut bb) { Some(bb) => break bb, _ => spin = spin + 1 } };

*/