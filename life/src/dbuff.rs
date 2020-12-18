//#![allow(dead_code)]

/// Delta Buffer.  Keeps track of last buffer for manual delta comparison

#[derive(Debug)]
pub struct Dbuff {
    pub buffa: Vec<i32>,
    pub buffb: Vec<i32>,
    pub tick: usize
}

impl Dbuff {

    // Put buffers in useable state.  Current buff is all 0, previous buff all MIN.
    pub fn new (len :usize) -> Dbuff {
        let mut ba = Vec::with_capacity(len);
        let mut bb = Vec::with_capacity(len);
        ba.resize(len, 0);
        bb.resize(len, ::std::i32::MIN);
        Dbuff {
            buffa: ba,
            buffb: bb,
            tick:  0
        }
    }

    pub fn state (&self) -> bool { 1 == self.tick & 1 }

    pub fn buff (&self) -> &Vec<i32> {
        if self.state() { &self.buffb } else { &self.buffa }
    }
    pub fn bufflast (&self) -> &Vec<i32> {
        if self.state() { &self.buffa } else { &self.buffb }
    }
    pub fn buffs (&self) -> (&Vec<i32>, &Vec<i32>) {
        if self.state() {
            (&self.buffb, &self.buffa)
        } else {
            (&self.buffa, &self.buffb)
        }
    }

    pub fn tick (&mut self) -> &Self {
        self.tick += 1;
        let buff = if self.state() { &mut self.buffb } else { &mut self.buffa };
        buff.clear();
        self
    }

    // Put/append i32s onto active buffer
    pub fn put (&mut self, v :&[i32]) -> &Self {
        let buff = if self.state() { &mut self.buffb } else { &mut self.buffa };
        buff.extend_from_slice(v);
        self
    }

    pub fn db (&self) -> &Self {
        println!("{:?}", self);
        self
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
    db       .put(&a1)           .db();
    db.tick(); db.put(&[10,20,30])   .db();
    db.tick(); db.put(&[100,200,300]).db();
    println!("{:?}", a1); println!("{:?}", a2); println!("{:?}", a3);
}

pub fn main() {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    test_dbuff();
}