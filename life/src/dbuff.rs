/// Delta Buffer.  Keeps track of last buffer for delta comparison

use piston_window::*;

/// 
#[derive(Debug)]
pub struct Dbuff {
    buffa: Vec<i32>,
    buffb: Vec<i32>,
    pub tick: usize // when tick is 0, buffa is brand new and buffb empty.
}

impl Dbuff {

    pub fn put (&mut self, v :&[i32]) -> &Self {
        match self.tick & 1 {
            0 => self.buffa.extend_from_slice(v),
            _ => self.buffb.extend_from_slice(v)
        }
        self
    }

    pub fn tick (&mut self) -> &mut Self {
        self.tick += 1;
        match self.tick & 1 {
            0 => self.buffa.clear(),
            _ => self.buffb.clear()
        }
        self
    }

    pub fn db (&self) -> &Self {
        println!("{:?}", self);
        self
    }

    pub fn dumpPiston (
        &self,
        writes: &mut usize,
        width  :usize,
        height :usize,
        context  :piston_window::Context,
        graphics :&mut G2d
    ) -> &Self {
        let (ba, bb) = 
            match self.tick & 1 {
                0 => (&self.buffa, &self.buffb),
                _ => (&self.buffb, &self.buffa)
            };
        let mut col=0;
        let mut row=0;
        //clear([0.0, 0.0, 0.0, 1.0], graphics);
        for i in 0..width*height {
            if ba[i] != bb[i] {
                *writes += 1;
                rectangle(
                    if 0 != ba[i] { [ 0.0, 0.0, 1.0, 1.0 ] } else { [ 0.0, 0.0, 0.0, 1.0 ] },
                    [ col as f64 * 6.0, row as f64 * 6.0,
                    6.0,                6.0],
                    context.transform,
                    graphics);
            }
            col += 1;
            if col == width { col = 0; row += 1; } 
        }
        return self;
    } // Dbuff::dumpPiston

    /// Pub DBuff in a "all elemtns are different" state
    pub fn new (len :usize) -> Dbuff {
        let ba = Vec::with_capacity(len);
        let mut bb = Vec::with_capacity(len);
        bb.resize(len, -1);
        Dbuff {
            buffa :ba,
            buffb :bb,
            tick : 0
        }
    }
}

/// //////////// Test bf: /////////////////
/// 
fn test_dbuff () {
    let a1 = [1,2,3];
    let a2 = [10,20,30];
    let a3 = [100,200,300];
    println!("{:?}", a1); println!("{:?}", a2); println!("{:?}", a3);
    let mut db = Dbuff::new(10);
    db.       put(&a1)      .db();
    db.tick().put(&[10,20,30])   .db();
    db.tick().put(&[100,200,300]).db();
    println!("{:?}", a1); println!("{:?}", a2); println!("{:?}", a3);
}

pub fn main() {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    test_dbuff();
}