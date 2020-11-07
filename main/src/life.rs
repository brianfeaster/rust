use ::std::sync::{Arc, Mutex};
use ::std::thread::{spawn, JoinHandle};
use ::std::time::{SystemTime};
use ::std::ops::{Range};

////////////////////////////////////////////////////////////////////////////////
/// TODO: message passing pipeline
///   Verify a thread crashing with a lock and subsequent threads receiving
///   invalid locks can communicate the new state (machine).
///

////////////////////////////////////////////////////////////////////////////////
/// Game of life state machine.  Let's try and make this concept happen.
///
struct State {
    power :bool,
    randomize :bool,
    key: String
}

impl State {
    pub fn next (self :&mut State,
                 tb :& crate::term::Tbuff) -> &State {
        self.randomize = false;
        self.key = tb.getc();
        match self.key.as_str() {
            "q" => self.power = false,
            " " => self.randomize = true,
            _ => ()
        }
        self
    }
    pub fn powered   (self :& State) -> bool { self.power }
    pub fn randomize (self :& State) -> bool { self.randomize }

    pub fn new () -> State {
        State {
            key  :"".to_string(),
            power :true,
            randomize :true
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
    let mut k = 0; // Column index
    // Sum of columns window
    let mut a = 0; // Not set initially
    let mut b = ra[w-1] + rb[w-1] + rc[w-1]; // Last column of game field
    let mut c = ra[k]   + rb[k]   + rc[k];   // First column of game field
    let firstCol = c;

    for j in 0..w { // Along the row
        k = k + 1; // next column index
        // Shift colums left
        a = b;
        b = c;
        c = if k==w { firstCol } else { ra[k] + rb[k] + rc[k] };
        let lives = a + b + c; // Window lives count
        // Set the next generation cell value
        bb[j] = (3 == lives || (4 == lives && 1 == rb[j])) as i32;
    }
}

fn genNewRows (
        aa      :& Arena,
        bb      :& Arena,
        range   :Range<usize>,
        h       :usize,
        w       :usize,
        threads :&mut Vec<JoinHandle<()>>) {
    let ra = aa.clone();
    let rb = aa.clone();
    let rc = aa.clone();
    let row = bb.clone();
    threads.push(spawn( move || {
        for i in range { // Over the rows...
            loop { // Lock the four rows and generate new row or retry...
                let ral = ra[(i+h-1) % h].try_lock(); if ral.is_err() { print!("!"); continue }
                let rbl = rb[i].try_lock();           if rbl.is_err() { print!("!"); continue }
                let rcl = rc[(i+1) % h].try_lock();   if rcl.is_err() { print!("!"); continue }
                let rowl = row[i].try_lock();         if rowl.is_err() { print!("!"); continue }
                genNewRow(w,
                    & ral.unwrap(),
                    & rbl.unwrap(),
                    & rcl.unwrap(),
                    &mut rowl.unwrap());
                break;
            }
        }
    }));
}

type Arena = Arc<Vec<Mutex<Vec<i32>>>>;

fn arena_render (
        aa :&Arena,
        tb :&mut crate::term::Tbuff) {
    for y in (0..tb.rows()).rev() {
        let row = aa[y].lock().unwrap();
        for x in 0..tb.cols() {
            if 0 != row[x] {
                tb.set(x, y, 4, 12, '◼') // ▪ ◾ ◼ ■ █
            } else {
               //tb.set(x, y, 3, 0, ' ')
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

const ARENA_HEIGHT :i32 = 423;
const ARENA_WIDTH :i32 = 1430;

fn arena_new () -> Arena  {
    let mut arena = Arc::new(vec![]);
    let rows = Arc::get_mut(&mut arena).unwrap();
    for y in 0..ARENA_HEIGHT {
        rows.push(
            Mutex::new(
                (y * ARENA_WIDTH .. y * ARENA_WIDTH + ARENA_WIDTH).map(|_| if 0 == crate::ri32(10) { 1 } else { 0 })
                    .collect::<Vec<i32>>()));
    }
    return arena;
}

fn life () {
    // Coneable objects that can be passed to threads
    let arena = arena_new();
    let arenb = arena_new();
    let arc_mut_tb = Arc::new(Mutex::new(crate::term::Tbuff::new()));

    let mut state = State::new();
    let mut tick = 0;
    let epoch = SystemTime::now(); // For FPS calculation
    let mut threads :Vec<JoinHandle<()>> = vec!();

    while state.powered() && tick < 10000 { // Loop until keypress 'q'

        //util::sleep(100);
        
        let (aa, bb) = // aa is the current arena (to read/dump), bb the next arena (to generate/overwrite)
            match 0 == tick & 1 {
                true  => (&arena, &arenb),
                false => (&arenb, &arena)
            };

        for _ in 0..threads.len() { threads.pop().unwrap().join().unwrap(); }

        let (w, h, z) = {
            let mut tb = arc_mut_tb.lock().unwrap();
            tb.reset(tick) // Need to be muteable
              .dims()
        };


        // Draw the arena in a separate thread.  There's no reason to join
        // on the thread since it has a lock on the Tbuff and will block
        // anyone wanting to write to it.
        // Todo: 
        // 
        // Holds a lock on TB which is needed at the end of this loop in main thread
        // Periodically lock on aa
        //threads.push({
        //    let aa = aa.clone();
        //    let m_tb = arc_mut_tb.clone();
        //    spawn(move || {
        //        let mut tb = m_tb.lock().unwrap(); // Does need to be muteable
        //        arena_render(&aa, &mut tb);
        //        tb.dump().flush();
        //        println!("\x1b[0H\x1b[0;35m FPS:{:7.2} F:{} \x1b[40m  ", 1000.0 * tick as f32 / epoch.elapsed().unwrap().as_millis() as f32, tick);
        //    })
        //});


        // Either randomize the visible field or compute next generation
        if state.randomize() {
            arena_randomize(bb, h, w);
        } else {
            //genNewRows(aa, bb, 0..h, h, w, &mut threads); // 11.7k

            //genNewRows(aa, bb, 0   .. h/2, h, w, &mut threads); // 6.6k
            //genNewRows(aa, bb, h/2 .. h,   h, w, &mut threads);

            //genNewRows(aa, bb, 0     .. h/3,   h, w, &mut threads); // 4.6k
            //genNewRows(aa, bb, h/3   .. h*2/3, h, w, &mut threads);
            //genNewRows(aa, bb, h*2/3 .. h,     h, w, &mut threads);

            genNewRows(aa, bb, h*0/4 .. h*1/4, h, w, &mut threads); // 3.8k
            genNewRows(aa, bb, h*1/4 .. h*2/4, h, w, &mut threads);
            genNewRows(aa, bb, h*2/4 .. h*3/4, h, w, &mut threads);
            genNewRows(aa, bb, h*3/4 .. h*4/4, h, w, &mut threads);

            //genNewRows(aa, bb, h*0/5 .. h*1/5, h, w, &mut threads); // 4.2k
            //genNewRows(aa, bb, h*1/5 .. h*2/5, h, w, &mut threads);
            //genNewRows(aa, bb, h*2/5 .. h*3/5, h, w, &mut threads);
            //genNewRows(aa, bb, h*3/5 .. h*4/5, h, w, &mut threads);
            //genNewRows(aa, bb, h*4/5 .. h*5/5, h, w, &mut threads);

        }

        //for _ in 0..threads.len() { threads.pop().unwrap().join().unwrap(); }

        tick = tick + 1;
        state.next(& arc_mut_tb.lock().unwrap()); // Needs to be mutable
    }
    arc_mut_tb.lock().unwrap().done(); // Doesn't need to be mutable
    println!("{}", epoch.elapsed().unwrap().as_millis());
}

pub fn main () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    self::life();
}

// Garbage

// Spin until lock acquired
//let mut spin = 0; // keep track of busy waiting/spinning
//let bb = loop { match Arc::get_mut(&mut bb) { Some(bb) => break bb, _ => spin = spin + 1 } };