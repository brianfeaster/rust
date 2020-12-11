#![allow(dead_code)]

/// Delta Buffer.  Keeps track of last buffer for manual delta comparison
#[derive(Debug)]
pub struct Dbuff {
    pub buffa: Vec<i32>,
    pub buffb: Vec<i32>,
    pub tick: usize // when tick is 0, buffa is brand new and buffb empty.
}

impl Dbuff {

    pub fn put (&mut self, v :&[i32]) -> &mut Self {
        match self.tick & 1 {
            0 => self.buffa.extend_from_slice(v),
            _ => self.buffb.extend_from_slice(v)
        }
        self
    }

    pub fn get (&self, last: usize) -> &Vec<i32> {
        match (self.tick + last) & 1 {
            0 => &self.buffa,
            _ => &self.buffb
        }
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

    /// Put both buffers in a "all elemtns are different" state
    pub fn new (len :usize) -> Dbuff {
        let mut ba = Vec::with_capacity(len);
        let mut bb = Vec::with_capacity(len);
        ba.resize(len, -1);
        bb.resize(len, -2);
        Dbuff {
            buffa: ba,
            buffb: bb,
            tick:  0
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