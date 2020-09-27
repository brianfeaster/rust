#![allow(dead_code, unused_variables, non_snake_case)]
use std::thread;
use crate::util::{self};
use crate::term::{Term};

fn fun_split () {
    println!("[fun_string] {:?}", String::from("long live  donut ").split(" "));
    let mut v = vec!(1); // A vector               [1]
    println!("v={:?}", v);
    v[0]=69;             // Mutate first position  [69]
    println!("v={:?}", v);
    let f = &mut v[0];   // Ref to 1st position
    *f=42;               // Mutate first position  [42]
    println!("v={:?}", v);
    v.push(222);        // Push element to vector  [42, 222]
    println!("v={:?}", v);
}

fn fun_tuples() {
    let t = (1,2);
    let s = (11,{println!("hi"); 22});
    println!("[tuples]  t= {:?}", t.1);
    println!("[tuples]  s= {:?}", loop { break if true { s.1 } else { t .1 }} );
}

fn fun_map(_term :&mut Term) {
    let mut r = 1..=2;
    println!("{:?}", &r); // Can't move this after the for loop
    for i in &mut r { println!("{:?}", i); }
    println!("{:?}", &r); // Can't move this after the for loop
    let v = 5;
    println!("map {:?}", (0..=v).map(|x| x / 2_i32).collect::<Vec<i32>>()); // type std::ops::RangeInclusive
    println!("iter {:?}", [0,1,2,3].iter().map(|x| x*x ).collect::<Vec<i32>>()); // type [i32; 2]
}


fn fun_write_non_block (term : & mut Term) {
    term.termblock();
    use std::io::Write;
    loop {
        match std::io::stdout().write(b"abcdefghijklmnopqrstuvwxyz") { // .as_bytes()
            Ok(o) => {  println!(" Ok{}", o); if o != 26 { break; }},
            Err(e) => { util::flush(); println!("Err{}", e); util::sleep(1000); util::flush(); }
        }
    }
   //||->Result<(), &str> {Err("no")?;Ok(())}().unwrap_err().as_bytes());
}

fn fun_getc_loop (term :& Term) {
    term.terminalraw();
    //term.termblock();
    let mut s :String = String::new();
    while s.len() != 1 || &s[0..1] != "q" {
        s = term.getc();
        println!("getc '{}' {}'", s, s.len());
        //if s.as_bytes()[0] as char == 'q' { break; }
        if 0 == s.len() { util::sleep(1000); }
    }
}

fn fun_thread (term :& mut Term) {
    thread::spawn(|| {
        let term = Term::new();
        fun_getc_loop(&term);
        println!("thread done.]");
    });
    let mut c=10;
    while 0 < c {
        println!("[c={}]", c);
        c -= 1;
        util::sleep(500);
    }
}

fn fun_iter () {
    let w = util::Walk::new(&[0.0,0.0], &[10.0,2.0]);
    //print!("{:?} ", w); println!("{:?}", w.next());
    println!("{:?}", w);
    println!("{:?}", w.collect::<Vec<[i32; 3]>>());
    //for l in w { print!("{:?}", l); }
    //for l in w { print!("{:?}", l); }
}

pub fn main() {
    let a = 4;
    let ref mut b = Box::new([-3,-2,-1]);
    (*b)[1]=2;
    println!("{} {:?} {}", a, b, module_path!());

    //let ref mut term = Term::new();
    //fun_split();
    //fun_tuples();
    //fun_map(term);
    //fun_write_non_block(term);
    //fun_getc_loop(term);
    //fun_thread(term);
    //fun_iter();
}