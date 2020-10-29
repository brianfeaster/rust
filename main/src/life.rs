use ::std::sync::{Arc, Mutex};

/// Update/mutate the next gen of life in row 'bb' given the current
/// row 'rb', the row above 'ra', and row below 'rc'.
fn genRow (ra :&[i32], rb :&[i32], rc :&[i32], bb :&mut [i32]) {
    let w = bb.len(); // width of row
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

fn render (aa :&[i32], tb :&mut crate::term::Tbuff) {
    let w = tb.cols();
    for (i, c) in aa.iter().enumerate() {
         if 0 != *c {
            tb.set(i%w, i/w, 4, 12, ' ') // ▪ ◾ ◼ ■ █
         } else {
            //tb.set((i%w) as i32, (i/w) as i32, 4, 11, '.')
         }
    }
}

struct State {
    power :bool,
    randomize :bool,
    key: String
}

impl State {
    pub fn next (self :&mut State,
                 tb :&mut crate::term::Tbuff) -> &State {
        self.randomize = false;
        self.key = tb.getc();
        match self.key.as_str() {
            "q" => self.power = false,
            " " => self.randomize = true,
            _ => ()
        }
        self
    }
    pub fn power     (self :& State) -> bool { self.power }
    pub fn randomize (self :& State) -> bool { self.randomize }

    pub fn new () -> State {
        State {
            key  :"".to_string(),
            power :true,
            randomize :true
        }
    }
}


fn life () {
    const Z :usize = 65536;
    let mut rc_auff :Arc<[i32; Z]> = Arc::new([0; Z]);
    let mut rc_buff :Arc<[i32; Z]> = Arc::new([0; Z]);
    let epoch = ::std::time::SystemTime::now(); // For FPS calculation
    let mut state = self::State::new();
    let mut tick  = 0;
    let mut spin = 0; // keep track of busy waiting/spinning

    // The Terminal Buffer needs to be mutexed
    let rc_m_tb =  Arc::new(Mutex::new(crate::term::Tbuff::new()));
    let tb_m = rc_m_tb.clone();

    while state.power() { // Loop until keypress 'q'

        let (w, h, z) = {
            let mut tb = tb_m.lock().unwrap();
            tb.reset(tick).dims()
        };

        // aa is the current arena (to read/dump), bb the next arena (to generate/overwrite)
        let (aa, bb) = // [0..z]
            match 0 == tick & 1 {
                true  => (Arc::clone(&rc_auff),
                          loop {
                              match Arc::get_mut(&mut rc_buff) {
                                  Some(bb) => break bb ,
                                         _ => spin = spin + 1 } }),
                false => (Arc::clone(&rc_buff),
                          loop {
                              match Arc::get_mut(&mut rc_auff) {
                                  Some(bb) => break bb,
                                         _ => spin = spin + 1 } })
            };

        // Draw the arena in a separate thread
        let tb_m = rc_m_tb.clone();
        let aa2 = aa.clone();
        let t1 = std::thread::spawn( move || {
        {
            let mut tb = tb_m.lock().unwrap();
            self::render(&aa2[0..z], &mut tb);
            tb.dump().flush();
        }
        });

        if state.randomize() {
            // Randomize the field instead of computing next generation
            for i in 0..z { bb[i] = if 0 == crate::ri32(10) { 1 } else { 0 } }
            //auff.iter_mut().map( |i :&mut i32| *i = if 0 == crate::ri32(10) { 1 } else { 0 } ).count();
        } else {
            let aa = &aa[0..z];
            // Compute next generation
            let mut ri = 0; // Consider 2nd row index
            // Initialize row references for life computation (last row, 1st, and 2nd)
            let mut ra = &aa[..]; // Not set now
            let mut rb = &aa[z-w .. z ]; // last row
            let mut rc = &aa[0   .. w ]; // first row
            let rfirst = rc;
            for i in 0..h { // Over all rows
                ri = ri + w; // Increment index
                // Shift rows up
                ra = rb;
                rb = rc;
                rc = &aa[if ri < z {
                            ri .. ri+w // Next row
                         } else {
                            0 .. w  // Wrap around to first row
                         }];
                self::genRow(ra, rb, rc, &mut bb[i*w .. i*w+w]);
            }
        }

        tick = tick + 1;
        println!("\x1b[HFPS:{:?}  tick:{}  spin:{}  ", tick as f32 / epoch.elapsed().unwrap().as_secs() as f32, tick, spin);
        let tb_m2 = rc_m_tb.clone();
        state.next(&mut tb_m2.lock().unwrap());

        drop(aa);
        drop(bb);
        t1.join().unwrap(); // Wait for rendering thread to finish
    }
    //tb_m.lock().unwrap().done();
}

pub fn main () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    self::life();
}