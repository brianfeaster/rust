/// Delta Buffer.  Keeps track of last buffer for delta comparison

use piston_window::*;

#[derive(Debug)]
pub struct Dbuff {
    pub buffa: Vec<i32>,
    pub buffb: Vec<i32>,
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