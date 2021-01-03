use ::std::sync::{Arc, Mutex}; // External
use ::std::thread::{spawn, JoinHandle};
use ::std::ops::{Range};
use ::rand::{*, rngs::*};
use crate::dbuff::*; // Local

/// Game of life

// TODO:
// Old Description
// First time this is called, it will be overwriting Arena B with Arena A's next gen and subsequenty copy the arena to dbuff B.
// IN the mean time, dbuff A can be read and rednered.
// Tick should represent which Arena is readable.  tick==0 implies Arena A is readable
// After first gen_next, tick should be incremented to indicated Arena B should be read

/// Random boolean integer 32 bit.  Returns 0 or 1 with probability 1/m.
pub fn rbi32(m: u32) -> i32 { ((thread_rng().next_u32() % m) == 0) as i32 }
//pub fn rf32() -> f32 { random::<f32>() }

pub fn pattern1 (x: i32, y: i32, s: i32) -> bool {
    (x*x + y*y) <= s && x.abs() == y.abs()
}
pub fn pattern2 (x: i32, y: i32, s: i32) -> bool {
    ((x*x + y*y) <= s) && (x == 0 || y == 0 || x.abs() == y.abs() )
}

// Arena ///////////////////////////////////////////////////////////////////////

type Arena = Arc<Vec<Mutex<Vec<i32>>>>;

fn arena_new (width: i32, height: i32, s: i32) -> Arena  {
    let mut rrr = <StdRng as SeedableRng>::seed_from_u64(1);
    Arc::new(
        (0..height).map(
            |_h| Mutex::new(
                (0..width)
                .map( |_w| if s == 0 { 0 } else { (0 == (rrr.gen::<u32>() % 10)) as i32 } ) // rbi32(10)
                .collect::<Vec<_>>()))
        .collect::<Vec<Mutex<Vec<_>>>>())
}

fn draw_glider (aa :&Arena, x :usize, y :usize) {
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
                if s == -1 { rbi32(10) } else { pattern2(xx, yy, s.pow(2)) as i32 }
             }
        }
    }
}

fn bi (n :i32) -> i32 { ( 0 < n) as i32 }
/// Update/mutate the next gen of life in row 'bb' given the current
/// row 'rb', the row above 'ra', and row below 'rc'.
fn gen_new_rows (
        w :usize,
        ra :& Vec<i32>,
        rb :& Vec<i32>,
        rc :& Vec<i32>,
        bb :&mut Vec<i32>) {
    // Sum of columns window
    let mut a; // Not set initially
    let mut b = bi(ra[w-1]) + bi(rb[w-1]) + bi(rc[w-1]); // Last column of game field
    let mut c = bi(ra[0])   + bi(rb[0])   + bi(rc[0]);   // First column of game field
    let first_col = c;
    let mut state = bi(rb[0]);
    let mut next_state = 0;
    let mut k = 0; // Column index

    for j in 0..w { // Along the row
        // Shift colums left
        k = k + 1; // next column index
        a = b;
        b = c;
        c = if k==w { first_col } else { next_state = rb[k]; bi(ra[k]) + bi(rb[k]) + bi(rc[k]) };
        // Set the next generation cell alive if neighbor count 3 or 2 and currently alive.
        // Derivation: Let neighbor_count=n and state 0=dead 1=alive:
        //   Consider equation (n+state)<<1 - state
        //   4+1<<1-1 = 9
        //   3+1<<1-1 = 7  *
        //   2+1<<1-1 = 5  *
        //   1+1<<1-1 = 3
        //   ...
        //   4+0<<1-0 = 8
        //   3+0<<1-0 = 6  *
        //   2+0<<1-0 = 4
        // When equation is 5,6,7 next gen is alive, so sub 5 and simply check value is < 3
        let nxt = (( a + b + c << 1) as u32).wrapping_sub((bi(state)+5)as u32) < 3;
        // The alive/dead states are split into:
        // dead:  0:dead    was dead
        //       -1:died    dead after 1/born
        //       -2:croaked dead after 2
        // alive: 1:born    alive for 1
        //        2:old     alive past 1
        bb[j] = match (state, nxt) {
            (-2,false) => 0, (-1,false) => 0, (0,false) => 0,
            (-2,true) => 1,  (-1,true) => 1,  (0,true) => 1,
            (1,true) => 2, (2,true) => 2,
                           (2,false) => -2,
            (1,false) => -1,
            (x,y) => {print!("WeirdState({},{}) ", x, y); 0},
        };
        state = next_state; // Consider next cell for next iteration
    }
}

fn spawn_gen_next_rows (
        aa      :&Arena,
        bb      :&Arena,
        range   :Range<usize>,
        w       :usize,
        h       :usize
) -> JoinHandle<()> {
    let aa = aa.clone();
    let bb = bb.clone();
    spawn( move || {
        //println!("spawn_gen_next_rows:: {:?} {:?}", range.start, range.end);
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
                gen_new_rows(w,
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
    pub whs: (usize, usize, usize), // width height size
    arenas: (Arena, Arena), // Nested mutable ADTs that can be passed to threads
    pub dbuffs: (Arc<Mutex<Dbuff>>, Arc<Mutex<Dbuff>>),// Piston is 2xbuffered so we need a dbuff for each
    pub dbuff_en: bool,
    pub tick: usize,
    randomize: i32, // Randomizes field on next generation
    clear: bool,
    pub threads: usize, // How many threads to spanw for each generation computation
    pub threadvec :Vec<JoinHandle<()>>,
}

impl Life {
pub fn new (w: usize, h: usize) -> Life {
    Life {
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
    }
} }

impl Life {
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

    pub fn gen_next (&mut self) ->  &Arc<Mutex<Dbuff>> {
        // Wait for last rendering
        for _ in 0..self.threadvec.len() { self.threadvec.pop().unwrap().join().unwrap(); }
        self.arena_xfer_dbuff();
        self.tick += 1;
        let (aa, bb) = // aa is the current arena (to read), bb the next arena (to overwrite)
            match self.state() {
                false => (&self.arenas.1, &self.arenas.0),
                true  => (&self.arenas.0, &self.arenas.1)
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
                spawn_gen_next_rows(aa, bb, range, w, h);
                self.threadvec.push(t);
            }
        }
        // Return the dbuff to render, opposite the dbuff being generated
        let ret = match !self.dbuff_en || self.state() {
            false => &self.dbuffs.1,
            true  => &self.dbuffs.0
        };
        ret
    }

    pub fn arena_xfer_dbuff (&mut self) {
        let ab = match self.state() { // ab is the arena that was just generated
            false => &self.arenas.0,
            true  => &self.arenas.1
        };
        let mut dd = match self.dbuff_en && self.state() {
            false => &self.dbuffs.0,
            true  => &self.dbuffs.1
        }.lock().unwrap(); 
        dd.tick(); // Tick double buffer, clears next active buffer
        // Copy each mutexed arena row to double-buffer
        for y in (0..ab.len()).rev() { // Reverse the indexing so as to only conflict with the nexst generation thread once in the vector.
            //dd.put(&ab[y].lock().unwrap());
            loop { match ab[y].try_lock() { // Spin lock acquire
                  Err(_) => { print!("L0"); ::util::sleep(1); continue } ,
                  Ok(t) => { dd.put(&t); break }
            } }
        }
    }

}
