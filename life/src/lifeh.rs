use ::std::sync::{Arc, Mutex};
use ::std::thread::{spawn, JoinHandle};
use ::std::ops::{Range};
use ::std::collections::{HashMap};
use ::rand::{*, rngs::*};
use crate::dbuff::*; // Local

pub const LIFE_TITLE :&str = "LifeHash";

type WHS = (usize, usize, usize);

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

pub type ArenaBase = HashMap<(i32,i32),i32>;
type Arena = Arc<Mutex<ArenaBase>>;

fn arena_new (width: i32, height: i32, s: i32) -> Arena {
    let mut rrr = <StdRng as SeedableRng>::seed_from_u64(1);
    let mut hm = HashMap::new();
    for h in 0..height {
        for w in 0..width {
            if s != 0 && (0 == (rrr.gen::<u32>() % 10)) {
                hm.insert( (w as i32, h as i32), 1);
            }
        }
    }
    Arc::new(Mutex::new(hm))
}

fn draw_glider (aa :&Arena, x :i32, y :i32) {
    let mut aa = aa.lock().unwrap();
    aa.insert((x+2,y+0), 1);
    aa.insert((x+2,y+1), 1);
    aa.insert((x+2,y+2), 1);
    aa.insert((x+1,y+2), 1);
    aa.insert((x+0,y+1), 1);
}

fn arena_clear (aa :&Arena, _w :i32, _h :i32) {
    aa.lock().unwrap().clear();
}

fn arena_randomize (bb :&Arena, w :i32, h :i32, s: i32) {
    let mut bb = bb.lock().unwrap();
    bb.clear();
    for y in 0..h {
        for x in 0..w {
            if s == -1 {
               bb.insert((x,y), rbi32(10));
            } else {
                let xx = x as i32 - w as i32 /2;
                let yy = y as i32 - h as i32 /2;
                if pattern2(xx, yy, s.pow(2)) {
                    bb.insert((x,y), 1);
                }
            }
        }
    }
}

fn _intointbool (n :i32) -> i32 { ( 0 < n) as i32 }

/// Update/mutate the next gen Arena of life in row 'bb' given the current Arena aa
fn gen_new (
    whs :WHS,
    aa  :Arena,
    bb  :Arena,
) {
    let w = whs.0;
    let h = whs.1;
    let aa = aa.lock().unwrap();
    let mut counts = HashMap::new();
    // Increment neighor counts to each neighbor and set location to alive
    for ((x,y),c) in aa.iter() {
        if *c != 0 {
            let xm = (*x-1).rem_euclid(w as i32);
            let ym = (*y-1).rem_euclid(h as i32);
            let xp = (*x+1).rem_euclid(w as i32);
            let yp = (*y+1).rem_euclid(h as i32);
            *counts.entry((xm  ,ym)).or_insert(0) += 2;
            *counts.entry((*x,  ym)).or_insert(0) += 2;
            *counts.entry((xp  ,ym)).or_insert(0) += 2;

            *counts.entry((xm  ,*y)).or_insert(0) += 2;
            *counts.entry((*x,  *y)).or_insert(0) += 1;
            *counts.entry((xp  ,*y)).or_insert(0) += 2;

            *counts.entry((xm  ,yp)).or_insert(0) += 2;
            *counts.entry((*x,  yp)).or_insert(0) += 2;
            *counts.entry((xp  ,yp)).or_insert(0) += 2;
        }
    }

    let mut bb = bb.lock().unwrap();
    bb.clear();
    for (k, c) in counts.iter() {
        if (*c >> 1) == 3 || *c == 5 { bb.insert(*k,1); }
    }
}

fn spawn_gen_next_rows (
    whs     :WHS,
    aa      :&Arena,
    bb      :&Arena,
    _range  :Range<usize>,
    _w      :usize,
    _h      :usize
) -> JoinHandle<()> {
    let aa = aa.clone();
    let bb = bb.clone();
    spawn( move || gen_new(whs, aa, bb) )
}


/*/ Life ////////////////////////////////////////////////////////////////////////
   Two arenas that write back and forth to each eacher  one row at a time:
             ,------< one or more threads
    Arena A  v  Arena B   
    [     ] --> [     ]  DbuffB [    ]
    [     ] --> [     ]
    [     ] --> [     ]  DbuffA [    ]
            \____________^  <--- single thread should happen first or interleave?
  
   Arena  Buff  Arena  Buff
    .     .       .     .
    @ ->  *       -     .
    .     .       @     @
    .     .       .     .
    .     .       .     .
*/
pub struct Life {
    pub whs: (usize, usize, usize), // width height size
    pub arenas: (Arena, Arena), // Nested mutable ADTs that can be passed to threads
    pub arena: Option<ArenaBase>,
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
            arenas: (
                arena_new(w as i32, h as i32, 1),
                arena_new(w as i32, h as i32, 0) ),
            arena: Some(ArenaBase::new()),
            dbuffs: (
                Arc::new(Mutex::new(Dbuff::new(w*h, (-99, -9)))),
                Arc::new(Mutex::new(Dbuff::new(w*h, (-99, -9))))),
            dbuff_en: true,
            tick: 0,
            randomize: 0,
            clear: false,
            threads: 1,
            threadvec: vec!()
        }
    }
    pub fn randomize (&mut self, s: i32) -> &mut Self { self.randomize=s; self }
    pub fn clear (&mut self) -> &mut Self { self.clear=true; self }
    pub fn add_glider (&self, x: i32, y: i32) -> &Self {
        let aa = if self.state() {&self.arenas.1} else {&self.arenas.0};
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
            arena_randomize(bb, self.whs.0 as i32, self.whs.1 as i32, self.randomize);
            self.randomize = 0;
        } else if self.clear {
            self.clear = false;
            arena_clear(bb, self.whs.0 as i32, self.whs.1 as i32);
        } else {
            let (w, h,_) = self.whs;
            for p in 0 .. self.threads {
                let range = h*p/self.threads .. h*(p+1)/self.threads;
                let t = spawn_gen_next_rows(self.whs, aa, bb, range, w, h);
                self.threadvec.push(t);
            }
        }
        // Return the dbuff to render, opposite the dbuff being generated
        self.dbuff()
    }

    pub fn dbuff (&self) -> &Arc<Mutex<Dbuff>> {
        match !self.dbuff_en || self.state() {
            false => &self.dbuffs.1,
            true  => &self.dbuffs.0
        }
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
        let aa = ab.lock().unwrap();
        if self.arena.is_none() {
            for y in (0..self.whs.1).rev() {
                let mut s = Vec::new(); // Build a row
                for x in 0..self.whs.0 {
                    s.push(
                        match aa.get(&(x as i32,y as i32)) {
                            None => 0,
                            Some(v) => if 0 < *v  { 1 } else { 0 }
                        }
                    );
                }
                dd.put(&s);  // Write row to buffer
            }
        } else {
            let arena = self.arena.as_mut().unwrap();
            arena.clear();
            for ((x,y),v) in aa.iter() { arena.insert((*x,self.whs.1 as i32 -*y), *v); }
        }
    }
} // impl Life

/*
hash table, no double-buff render optimization
1 [39.17]       gens:79 ∆lifes:1756 spins:0 s=1
1 [50.24]       gens:180 ∆lifes:1682 spins:0 s=1
1 [49.87]       gens:280 ∆lifes:1878 spins:0 s=1
1 [48.17]       gens:377 ∆lifes:1729 spins:0 s=1
1 [44.60]       gens:467 ∆lifes:1848 spins:0 s=1
1 [48.39]       gens:564 ∆lifes:1790 spins:0 s=1

 2x2 array
1 [16.89]       gens:34 ∆lifes:1240 spins:0 s=1
1 [21.83]       gens:78 ∆lifes:970 spins:0 s=1
1 [22.53]       gens:124 ∆lifes:1113 spins:0 s=1
1 [22.90]       gens:170 ∆lifes:937 spins:0 s=1
1 [22.76]       gens:216 ∆lifes:1072 spins:0 s=1
*/