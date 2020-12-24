#![allow(non_snake_case)]
use std::collections::{HashMap, HashSet};
use regex::Regex;
use std::fs::{read_to_string};
use std::fmt::{Debug};
use util::{Plotter};
use std::ops::{Index};

////////////////////////////////////////////////////////////////////////////////
// Useful

fn db<T: Debug> (o: &T) { println!("{:?}", o); }

/// Strings Table
#[derive (Debug)]
struct Strings {
    hm: HashMap<usize, String>
}
impl Strings {
    fn new () -> Strings  {
        Strings {
            hm: vec![(std::usize::MAX, "".to_string())]
                .drain(..).collect()
        }
    }
    //fn put (&mut self, i: usize, s: &str) { self.hm.insert(i, s.to_string()); }
}
impl Index<usize> for Strings {
    type Output = String;
    fn index(&self, num: usize) -> &Self::Output {
        self.hm.get(
            &if self.hm.contains_key(&num) { num } else { std::usize::MAX }
        ).unwrap()
    }
}
//impl IndexMut<usize> for Strings {
//    fn index_mut(&mut self, num: usize) -> &mut Self::Output {
//        self.hm.entry(num).or_insert(String::new())
//    }
//}

////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
// Day 1
type Hm = HashMap<i32, i32>;

fn numbers_to_hash ( inputstr: String) -> Hm {
    let mut the_hash_map = HashMap::new();
    inputstr.lines().for_each( |s| {
        match s.parse::<i32>() {
            Ok(i) => {
                match the_hash_map.insert(i, i) {
                    None => (),
                    e => println!("ERROR: overwriting hashtable with {} {:?}", i, e)
                }
            },
            _ =>  println!("\x1b[1;31mERROR: can't parse integer from {}\x1b[0m", s)
        }
    });
    the_hash_map
}

/// Find pairs in hash table that add to 2020, compute product
fn find2020(ht: &Hm) {
    for h in ht {
        let p = h.0;
        let diff = 2020-p;
        if let Some(q) = ht.get(&diff) {
            println!("{} + {} = {}, multiplied = {}", p, q, p+q, p*q);
        }
    }
}

/// Find triplets in hash table that add to 2020, compute product
fn find2020x3(ht: &Hm) {
    for h in ht {
        for i in ht {
            let p = h.0;
            let q = i.0;
            let diff = 2020-p-q;
            if let Some(r) = ht.get(&diff) {
                println!("{} + {} + {} = {}, multiplied = {}", p, q, r, p+q+r, p*q*r);
            }
        }
    }
}

fn day1 () {
    ::std::println!("== {}:{} ::{}::day1() ====", std::file!(), core::line!(), core::module_path!());
    match ::std::fs::read_to_string("data/input1.day") {
        Ok(filestr) => {
            let ht = numbers_to_hash(filestr);
            find2020(&ht);
            find2020x3(&ht);
        },
        e => println!("Unable to readfile {:?}", e)
    }
}
// Day 1
////////////////////////////////////////////////////////////////////////////////
// Day 2
// 
fn validate_passwords (filename: &str) -> usize {
    Regex::new(r"(\d+)-(\d+) (.): (.*)").unwrap()
    .captures_iter(&::std::fs::read_to_string(filename).unwrap())
    .map( |cap| {
        let from = cap[1].parse::<usize>().unwrap();
        let to   = cap[2].parse::<usize>().unwrap();
        let thech = &cap[3];
        let thepw = &cap[4];
        let pwchcount = thepw.chars().filter( |c| c.to_string() == thech).count();
        (from, to, pwchcount)
    })
    .filter(|(from, to, count)| from <= count && count <= to)
    .count()
}

fn validate_passwords2 (filename: &str) -> usize {
    Regex::new(r"(\d+)-(\d+) (.): (.*)")
    .unwrap()
    .captures_iter(&::std::fs::read_to_string(filename).unwrap())
    .map( |cap| {
        let first = cap[1].parse::<usize>().unwrap() - 1; // Pascal programmers
        let second = cap[2].parse::<usize>().unwrap() - 1;
        let ch = &cap[3].as_bytes()[0];
        let pw = &cap[4].as_bytes();
        first <= pw.len()-1 &&
        second <= pw.len()-1 &&
        (&pw[first] == ch) ^ (&pw[second] == ch)
    })
    .filter( |s| *s)
    .count()
}

fn day2 () {
    ::std::println!("== {}:{} ::{}::day2() ====", std::file!(), core::line!(), core::module_path!());
    println!("Valid passwords: {}", validate_passwords("data/input2.day"));
    println!("Valid passwords 2nd approach: {}", validate_passwords2("data/input2.day"));
}

// Day 2
////////////////////////////////////////////////////////////////////////////////
// Day 3

fn walks (filename: &str, dirs: &[(usize, usize)]) -> usize {
    dirs.iter()
    .map( |(dx, dy)|
        ::std::fs::read_to_string(filename).unwrap().lines()
        .step_by(*dy)
        .enumerate()
        .filter(|(i, line)| '#' == line.chars().cycle().nth(i * *dx).unwrap())
        .count())
    .product::<usize>()
}
fn day3 () {
    ::std::println!("== {}:{} ::{}::day3() ====", std::file!(), core::line!(), core::module_path!());
    vec![ vec![(3, 1)], vec![(1,1),(3,1),(5,1),(7,1),(1,2)] ]
    .iter()
    .for_each( |dirs| println!("Result = {:?} for {:?}", walks("data/input3.txt", dirs), dirs) );
}

// Day 3
////////////////////////////////////////////////////////////////////////////////
// Day 4

type Hm4 = HashMap<String, String>;

fn scan_passport (filename: &str) -> Vec<Hm4> {
    ::std::fs::read_to_string(filename).unwrap().lines()
    .fold(
        vec![HashMap::new()], // Initial vector contains empty hash table
        |mut v, line| {
            line.split(' ') // ["key:val", ...] or [""] when the line is empty
            .for_each( |kvstr| { // "key:val" or ""
                if kvstr == "" {
                    v.insert(0, HashMap::new()) // Start a new hash table
                } else {
                    // Add k/v to hashtable at head of vector
                    let kv = kvstr.split(':').collect::<Vec<&str>>();
                    v[0].insert(kv[0].to_string(), kv[1].to_string());
                }
            });
            v // Return the vector (is this copied each itme?) for next fold iteration
        }
    ) // fold
}

fn validate_passport1 (h: &Hm4) -> bool {
    h.len() == 8 || (h.len()==7 && None == h.get("cid"))
}

fn validate_passport2 (h: &Hm4) -> bool {
    let s = &"0".to_string(); // Bogus string
    validate_passport1(h)
    && {
        let byr = h.get("byr").unwrap_or(s).parse::<i32>().unwrap();
        1920 <= byr && byr <= 2002
    } && {
        let iyr = h.get("iyr").unwrap_or(s).parse::<i32>().unwrap();
        2010 <= iyr && iyr <= 2020
    } && {
        let eyr = h.get("eyr").unwrap_or(s).parse::<i32>().unwrap();
        2020 <= eyr && eyr <= 2030
    } && {
        let cap = Regex::new(r"(\d+)(.*)").unwrap().captures(h.get("hgt").unwrap_or(s)).unwrap();
        let h = cap[1].parse::<i32>().unwrap();
        let u = &cap[2];
        (u=="cm" && (150 <= h && h <= 193)) ||
        (u=="in" && (59  <= h && h <= 76))
    } && {
        Regex::new(r"^#[0-9a-f]{6}$").unwrap().captures(h.get("hcl").unwrap_or(s)).is_some()
    } && {
        Regex::new(r"^(amb|blu|brn|gry|grn|hzl|oth)$").unwrap().captures(h.get("ecl").unwrap_or(s)).is_some()
    } && {
        Regex::new(r"^\d{9}$").unwrap().captures(h.get("pid").unwrap_or(s)).is_some()
    }
}

fn day4 () {
    ::std::println!("== {}:{} ::{}::day4() ====", std::file!(), core::line!(), core::module_path!());
    println!("Valid passports v1 {:?}",
        scan_passport("data/input4.txt").iter().filter( |h| validate_passport1(*h) ).count());
    println!("Valid passports v2 {:?}",
        scan_passport("data/input4.txt").iter().filter( |h| validate_passport2(*h) ).count());
}
// Day 4
////////////////////////////////////////////////////////////////////////////////
// Day 5

fn readbinaries (filename: &str) -> usize {
   *::std::fs::read_to_string(filename).unwrap() // Read file
     // Fix binary digits
    .chars().map( |c| match c { 'B'|'R'=>'1', 'F'|'L'=>'0', _=>c} )
    .collect::<String>().lines()
    // Parse binary numbers
    .map( |s| usize::from_str_radix(&s, 2).unwrap() )
    .fold( // Create the plane, fill with vacancies, then remove them
        (0..128) // New rows
        .map( |r| (r*8 .. (r+1)*8) .collect::<HashSet<usize>>()) // New seats
        .collect::<Vec<HashSet<usize>>>(), // New plane
        |mut t, d| { t[d/8].remove(&d); t } ) // Remove vacancy
    .iter().filter( |v| v.len() == 1 ) // Last available row
    .nth(0).unwrap().iter().nth(0).unwrap() // Last available seat
}

fn day5 () {
    ::std::println!("== {}:{} ::{}::day5() ====", std::file!(), core::line!(), core::module_path!());
    println!("Your seat number: {}", readbinaries("data/input5.txt")); // 524
}
// Day 5
////////////////////////////////////////////////////////////////////////////////
// Day 6

fn read_input_6 (filename: &str) -> Vec<HashMap<char, usize>> {
    let mut v = vec![HashMap::new()];
    for line in ::std::fs::read_to_string(filename).unwrap().lines() {
        if 0 == line.len() { // Start a new hash table
            v.insert(0, HashMap::new());
        } else { // increment person '#' and letter counts
            *(v[0].entry('#').or_insert(0)) += 1;
            line.chars().for_each( |ch| *(v[0].entry(ch).or_insert(0)) += 1 );
        }
    }
    v
}

// Sum how man questions were answered (hash
// entires), subtracting the passenger hash entry.
fn day6a (h: &Vec<HashMap<char,usize>>) -> usize {
    h.iter().map( |h| h.len() - 1 ).sum::<usize>()
}

// Sum how many questions were answered by all people in each
// group (same count), subtracting passenger hash entry.
fn day6b (h: &Vec<HashMap<char,usize>>) -> usize {
    h.iter().map( |h| {
        let nump = h.get(&'#').unwrap();
        h.values().filter( |v| *v == nump ).count() - 1
    }).sum()
}

fn day6 () {
    ::std::println!("== {}:{} ::{}::day6() ====", std::file!(), core::line!(), core::module_path!());
    let h = read_input_6("data/input6.txt");
    println!("Result A: {:?}", day6a(&h));
    println!("Result B: {:?}", day6b(&h));
}
// Day 6
////////////////////////////////////////////////////////////////////////////////
// Day 7
type H7 = HashMap<String, HashMap<String, usize>>;

fn read7 (filename: &str) -> H7 {
    Regex::new(r"(.*) bags contain (.*)\.").unwrap()
    .captures_iter(&::std::fs::read_to_string(filename).unwrap())
    .map( |cap| (
            cap[1].chars().collect::<String>(), // key/bag
            cap[2].chars().collect::<String>()  // Map of bag counts
            .split(", ").filter( |sub| sub != &"no other bags" )
            .map( |sub| Regex::new(r"(\d+) (.*) bags?").unwrap().captures(&sub).unwrap() )
            .map( |cap|
                    (cap[2].to_string(), // key/bag
                     cap[1].parse::<usize>().unwrap())) // count
            .collect::<HashMap<_,_>>() ) )
    .collect::<HashMap<_,_>>()
}

fn find7a<'a> (h: &'a H7, n: &str) -> HashSet<&'a str> {
    h.iter()
    .filter( |(_,v)| v.get(n).is_some() ) // Find parent bags containing n
    .fold(
        HashSet::new(),
        |mut s, (k,_)| {
            s.insert(k); // Keep track of n's parent bags
            find7a(h, k).iter().for_each( |k| { s.insert(k); } ); // Gather their parents, etc
            s
        }
    )
}

fn doit7a (filename: &str) -> usize {
    find7a(&read7(filename), &"shiny gold")
    .iter()
    .count()
}

fn find7b<'a> (h: &'a H7, mem: &mut HashMap<String, usize>, b: &str) -> usize {
    if let Some(v) = mem.get(b) {
        *v // Memoized sub-bag count
    } else {
        let sum = h.get(b).unwrap().iter().map( |(k,v)| v * (1 + find7b(h, mem, k))).sum();
        mem.insert(b.to_string(), sum); // Memoize bag count
        sum
    }
}

fn doit7b (filename: &str) -> usize {
    find7b(&read7(filename), &mut HashMap::new(), "shiny gold")
}

fn day7 () {
    ::std::println!("== {}:{} ::{}::day7() ====", std::file!(), core::line!(), core::module_path!());
    println!("Result A: {:?}", doit7a("data/input7.txt"));
    println!("Result B: {:?}", doit7b("data/input7.txt"));
}

// Day 7
////////////////////////////////////////////////////////////////////////////////
// Day 8
#[derive (Debug)]
enum Op { NOP, ACC, JMP, XXX }

#[derive (Debug)]
struct Inst { inst: Op, val: i32, dirty: bool }

type Prog = Vec<Inst>;

fn clear8 (prog: &mut Prog) {
    for inst in prog { inst.dirty = false; }
}

fn parsefile8 (thefile: &str) -> Prog {
    thefile.lines()
    //.inspect( |l| println!("LINE {}", l) )
    .map( |l| l.split(" ").collect::<Vec<_>>() )
    .map( |v|
        Inst{
            inst:  match &v[0] { &"nop" => Op::NOP, &"acc"=>Op::ACC, &"jmp"=>Op::JMP, _=>Op::XXX },
            val:   v[1].parse::<i32>().unwrap(),
            dirty: false} )
    .collect::<Prog>()
}

fn run8 (prog: &mut Prog, mut ip: usize, errorondirty: bool) -> Option<i32> {
    let mut acc :i32 = 0;
    while {//print!("[{}] ", ip);
           ip < prog.len()}
          && {// println!("{:?} {:?}", prog[ip].inst, prog[ip].val);
              true } {
        if prog[ip].dirty {
             //println!("ERROR: Cycle! acc={} Halting.", acc);
             return if errorondirty { None } else { Some(acc) }
        }
        prog[ip].dirty = true;
        let inst :&Op  = &prog[ip].inst;
        let val  :i32 = prog[ip].val;
        
        match inst {
            Op::ACC => { ip += 1; acc += val; },
            Op::JMP => { ip = (ip as i32  + val) as usize; },
            Op::NOP => { ip += 1; }
            Op::XXX => { println!("XXX"); return None; }
        }
    }
    if ip == prog.len() {
        println!("");
        Some(acc) // Program went past memory, The only good case.
    } else {
        //println!("\nERROR: Went over last instruction.");
        None
    }
}

fn doit8b (thefile: &str) -> Option<i32> {
    let mut prog = parsefile8(thefile);
    //run8(&mut prog, 0) // 643 last instruction
    for i in 0 .. prog.len() {
        match prog[i].inst {
        Op::JMP  => {
            //println!("Replacing [{}] {:?} {:?} to NOP", i, prog[i].inst, prog[i].val);
            prog[i].inst = Op::NOP;
            if let Some(ret) = run8(&mut prog, 0, true) { return Some(ret); }
            prog[i].inst = Op::JMP;
            clear8(&mut prog);
        },
        Op::NOP => {
            //println!("Replacing [{}] {:?} {:?} to JMP", i, prog[i].inst, prog[i].val);
            prog[i].inst = Op::JMP;
            if let Some(ret) = run8(&mut prog, 0, true) { return Some(ret); }
            prog[i].inst = Op::NOP;
            clear8(&mut prog);
        }
        _ => {}
        }
    }
    None
}

fn doit8a (thefile: &str) -> Option<i32> {
    let mut prog = parsefile8(thefile);
    run8(&mut prog, 0, false) // 643 last instruction
}

fn day8 () {
    println!("== {}:{} ::{}::day8() ====", std::file!(), core::line!(), core::module_path!());
    let thefile = read_to_string(&"data/input8.txt").unwrap_or("0:a\n1:b".to_string());
    println!("Result 8a: {:?}", doit8a(&thefile));
    println!("Result 8b: {:?}", doit8b(&thefile));
}
// Day 8
////////////////////////////////////////////////////////////////////////////////
// Day 9
type Adt9 = Vec<i64>;

fn parse9 (filename: &str) -> Adt9 {
    read_to_string(filename).unwrap_or("".to_string())
    .lines()
    .map( |e| e.parse::<i64>().unwrap() )
    //.inspect( |e| println!("<< {}", e) )
    //.count()
    .collect::<Vec<i64>>()
}

fn find_sum9(v: &[i64], pin: &i64) -> bool {
    v.iter().enumerate()
    .any( |(i, n)|
        v.iter()
        .skip(i)
        .any( |m| *pin == n + m ) )
}

fn doit9a (data: &Adt9) -> i64 {
   *data.iter()
    .skip(25).enumerate()
    .filter( |(i, e)| !find_sum9(&data[*i..*i+25], *e) ) // Find e that doesn't match
    .inspect( |(_, e)| println!("<< {}", *e) )
    .nth(0).unwrap().1
}

fn doit9b (data: Adt9, pin: i64) -> i64 {
    for i in 0..data.len() {
        for j in 2..data.len()+1 {
            let sum = data.iter().skip(i).take(j).sum();
            if pin < sum { break }
            if pin == sum {
                 let mut sorted = data.iter().skip(i).take(j).collect::<Vec<_>>();
                 sorted.sort();
                 return sorted[0] + sorted[sorted.len()-1];
            }
        }
    }
    return 0
}

fn day9 () {
    ::std::println!("== {}:{} ::{}::day9() ====", std::file!(), core::line!(), core::module_path!());
    let data = parse9("data/input9.txt");
    let num = doit9a(&data);
    println!("Result 9a: {:?}", num);
    println!("Result 9b: {:?}", doit9b(data, num));
}
// Day 9
////////////////////////////////////////////////////////////////////////////////
// Day10 
type Adt10 = Vec<usize>;

fn parse10 (filename: &str) -> Adt10 {
    read_to_string(filename).unwrap()
    .lines()
    .map( |e| e.parse::<usize>().unwrap())
    .collect::<Adt10>()
}

fn doit10a (data: &[usize], last: usize) -> usize {
    if 0 == data.len() {
        1 << (3-1) * 8
    } else {
        let num = data[0];
        doit10a(&data[1..], num) + (1 << (num-last-1)*8)
    }
}

fn doit10b (data: &[usize], last :usize, mem: &mut HashMap<String, usize>) -> usize {
    let key = format!("{}{:?}", last, data);
    if let Some(v) = mem.get(&key) { return *v } // Cached?
    let num = data[0];
    if 3 < num-last {
       0
    } else if data.len() == 1 {
       1
    } else {
        let res = doit10b(&data[1..], num, mem) + doit10b(&data[1..], last, mem);
        mem.insert(key, res);
        res
    }
}

fn day10 () {
    ::std::println!("== {}:{} ::{}::day10() ====", std::file!(), core::line!(), core::module_path!());
    let mut data = parse10("data/input10.txt");
    data.sort();

    let res = doit10a(&data, 0);
    println!("Result 10a: res = {} {} product = {:?}", res>>16, res%256, (res>>16) * (res%256));

    data.push(3 + data[data.len()-1]); // Append mine
    let mut mem = HashMap::new();
    let res = doit10b(&data, 0, &mut mem);
    println!("Result 10b: {:?}", res);
}

// Day 10
////////////////////////////////////////////////////////////////////////////////
// Day11 
type B11 = HashMap<(i32,i32),i32>;

fn parsefile11 (filename: &str) -> B11 {
    read_to_string(filename).unwrap().lines().enumerate()
    .map( |(y, l)| {
        l.chars().enumerate()
        .map( |(x,c)| {
            ((x as i32 ,y as i32 ), match c { '.'=>1, 'L'=>2, '#'=>3, _=>0 } as i32)
        }).collect::<Vec<_>>()
    }).flatten()
    .collect::<B11>()
}

fn counts11 (data:&B11) -> (i32, i32, i32) {
    data.iter()
    .fold( (0,0,0), |r,h|
     match h.1 {
         1=>(r.0+1, r.1,   r.2),
         2=>(r.0,   r.1+1, r.2),
         3=>(r.0  , r.1,   r.2+1),
         _=>r
    })
}



fn neighbors11a (data:&B11, x:i32, y:i32) -> i32 {
    0 +
    match data.get(&(x+1,y))   { Some(x) => if 3 == *x { 1 } else { 0 }, _ => 0 } +
    match data.get(&(x-1,y  )) { Some(x) => if 3 == *x { 1 } else { 0 }, _ => 0 } +
    match data.get(&(x  ,y+1)) { Some(x) => if 3 == *x { 1 } else { 0 }, _ => 0 } +
    match data.get(&(x  ,y-1)) { Some(x) => if 3 == *x { 1 } else { 0 }, _ => 0 } +

    match data.get(&(x+1,y+1)) { Some(x) => if 3 == *x { 1 } else { 0 }, _ => 0 } +
    match data.get(&(x-1,y+1)) { Some(x) => if 3 == *x { 1 } else { 0 }, _ => 0 } +
    match data.get(&(x-1,y-1)) { Some(x) => if 3 == *x { 1 } else { 0 }, _ => 0 } +
    match data.get(&(x+1,y-1)) { Some(x) => if 3 == *x { 1 } else { 0 }, _ => 0 }
}

fn next_gen11a (h:&B11) -> B11 {
    h.iter()
    .map( |((x,y), v)|
        match v {
            2 => if 0 == neighbors11a(&h, *x,*y) { ((*x, *y),3) } else { ((*x, *y),2) },
            3 => if 4 <= neighbors11a(&h, *x,*y) { ((*x, *y),2) } else { ((*x, *y),3) },
            _ => ((*x,*y),*v)
        })
    .collect::<B11>()
}

fn day11a (h: &B11) -> i32 {
    let mut counts = counts11(h);
    let mut next :B11 = next_gen11a(h); 
    loop {
        let counts2 = counts11(&next);
        if counts2 == counts { break counts.2 }
        counts = counts2;
        next = next_gen11a(&next);
    }
}


fn newcoor2b (x:i32, y:i32, d:usize) -> (i32, i32) {
   match d {
       0 => (x+1,y  ),
       1 => (x+1,y+1),
       2 => (x  ,y+1),
       3 => (x-1,y+1),
       4 => (x-1,y  ),
       5 => (x-1,y-1),
       6 => (x  ,y-1),
       7 => (x+1,y-1),
       _ => (x,y)
   }
}

fn occupieddir11b (h:&B11, x:i32, y:i32, d:usize) -> i32 {
    let mut c = (x,y);
    loop {
       c = newcoor2b(c.0, c.1, d);
       match h.get(&c) {
           Some(v) =>
            match *v {
                3 => break 1, // found a body
                2 => break 0, // found seat
                 _ => ()
            }, 
            None => break 0 // found edge
       }
    }
}

fn countseats11b (h:&B11, x:i32, y:i32) -> i32 {
    occupieddir11b(h, x,y, 0) + 
    occupieddir11b(h, x,y, 1) + 
    occupieddir11b(h, x,y, 2) + 
    occupieddir11b(h, x,y, 3) + 
    occupieddir11b(h, x,y, 4) + 
    occupieddir11b(h, x,y, 5) + 
    occupieddir11b(h, x,y, 6) + 
    occupieddir11b(h, x,y, 7)
}

fn next_gen11b (h:&B11) -> B11 {
    h.iter()
    .map( |((x,y), v)|
        match v {
            2 => if 0 == countseats11b(&h, *x,*y) { ((*x, *y),3) } else { ((*x, *y),2) },
            3 => if 5 <= countseats11b(&h, *x,*y) { ((*x, *y),2) } else { ((*x, *y),3) },
            _ => ((*x,*y),*v)
        })
    .collect::<B11>()
}


fn day11b (h: &B11) -> i32 {
    let mut counts = counts11(h);
    let mut next :B11 = next_gen11b(h); 
    loop {
        let counts2 = counts11(&next);
        if counts2 == counts { break counts.2 }
        counts = counts2;
        next = next_gen11b(&next);
    }
}

fn day11c(h: &B11, pltr: &mut Plotter) {
    pltr.renderhash(&h);
    let mut counts = counts11(&h);
    let mut hh = next_gen11a(h);
    loop {
        if pltr.renderhash(&hh).iskey('q') { return }
        let counts2 = counts11(&hh);
        if counts == counts2 { break } else { counts = counts2 }
        hh = next_gen11b(&hh);
    }
}

fn day11 () {
    ::std::println!("== {}:{} ::{}::day11() ====", std::file!(), core::line!(), core::module_path!());

    let mut pltr = Plotter::new();
    pltr.color(1,[0.0, 0.0, 0.0, 1.0]).color(2,[0.0, 0.0, 5.0, 1.0]).color(3,[0.5, 0.5, 0.5, 1.0]);

    let h :B11 = parsefile11("data/input11.txt");
    println!("Result 11a: {:?}", day11a(&h)); // 2113
    println!("Result 11b: {:?}", day11b(&h)); // 1865
    day11c(&h, &mut pltr);
}
// Day11 
////////////////////////////////////////////////////////////////////////////////
// Day 12
type B12 = Vec<(String,i32)>;

fn parse12 (filename: &str) -> B12 {
    Regex::new(r"([a-zA-Z]+)(\d+)").unwrap()
    .captures_iter(&::std::fs::read_to_string(filename).unwrap())
    .map( |cap| {
        let dir = cap[1].to_string();
        let amt = cap[2].parse::<i32>().unwrap();
        (dir, amt)
    })
    .collect::<B12>()
}

fn dirtodeltaadd (dir:i32, delta:i32, x:i32, y:i32) -> (i32, i32) {
    match dir.rem_euclid(4) {
        0 => (delta+x, 0+y),
        1 => (0+x, delta+y),
        2 => (-delta+x, 0+y),
        3 => (0+x, -delta+y),
        _ => (x, y)
    } 
}

fn doit12a (data: &B12) -> i32 {
    let mut pltr = ::util::Plotter::new();
    let res =
    data.iter()
    //.inspect( |(a,d)| print!("{}/{} ", a,d) )
    .fold( (0,0,0), | (mut d,mut x,mut y), (cmd, delta) | {
        match &cmd[..] {
            "R" => d -= match delta { 90=>1, 180=>2, 270=>3, _=>0 },
            "L" => d += match delta { 90=>1, 180=>2, 270=>3, _=>0 },
            "F" => { let xy = dirtodeltaadd(d,  *delta, x, y);  x = xy.0;  y = xy.1; },
            "N" => { let xy = dirtodeltaadd(1,  *delta, x, y);  x = xy.0;  y = xy.1; },
            "S" => { let xy = dirtodeltaadd(3,  *delta, x, y);  x = xy.0;  y = xy.1; },
            "E" => { let xy = dirtodeltaadd(0,  *delta, x, y);  x = xy.0;  y = xy.1; },
            "W" => { let xy = dirtodeltaadd(2,  *delta, x, y);  x = xy.0;  y = xy.1; },
            _ => ()
        }
        pltr.insert(x/10, y/10, 7).render();
        (d, x, y)
    });
    res.1.abs() + res.2.abs()
}

fn doit12b (data: &B12) -> i32 {
    let mut pltr = ::util::Plotter::new();
    let res =
    data.iter()
    .fold( (10,1,0,0),
     | (mut wx, mut wy, mut x,mut y), (cmd, delta) | {
        match &cmd[..] {
            "F" => { x+=wx*delta;  y+=wy*delta; },

            "L" => match delta {
               90 => { let yy=wx; let xx=-wy;  wx=xx; wy=yy; },
              180 => { wx=-wx;  wy=-wy; },
              270 => { let yy=-wx; let xx=wy;  wx=xx; wy=yy; },
              _ => () },

            "R" => match delta {
               90 => { let yy=-wx; let xx=wy;  wx=xx; wy=yy; },
              180 => { wx=-wx;  wy=-wy; },
              270 => { let yy=wx; let xx=-wy;  wx=xx; wy=yy; },
              _ => () },

            "N" => { wy += delta },
            "S" => { wy -= delta },
            "E" => { wx += delta },
            "W" => { wx -= delta },
            _ => ()
        };
        pltr.insert(x/100, y/100, 7).render();
        (wx, wy, x, y)
    } );
    res.2.abs() + res.3.abs()
}


fn day12 () {
    ::std::println!("== {}:{} ::{}::day12() ====", std::file!(), core::line!(), core::module_path!());
    let data = parse12("data/input12.txt");
    println!("Parse 12a: {:?}", doit12a(&data));
    println!("Parse 12b: {:?}", doit12b(&data));
    //println!("Result Jb: {:?}", parse12("data/input12.txt"));
}
// Day 12
////////////////////////////////////////////////////////////////////////////////
// Day 13
type B13 = (i32, Vec<i32>);

fn parse13 (filename: &str) -> B13 {
    let lines =
        read_to_string(filename)
        .unwrap_or_else( |err| {println!("{:?}", err); err.to_string()})
        .lines()
        .map( |l| l.to_string() )
        .collect::<Vec<String>>();

    (   lines[0].parse::<i32>().unwrap()
     ,
        lines[1].split(",")
        .filter_map( |e| e.parse::<i32>().ok() )
        .collect::<Vec::<i32>>() )
}

fn doit13a (data: &B13) -> i32 {
    let (t0, bs) = data;
    bs.iter()
    .map( |b| vec!(*b, *b - (t0 % *b) ) )  // bus# and  remaining time until arrival
    .min_by( |x,y| x[1].cmp(&y[1]) )
    .unwrap().iter().product()
} // 863 * 5 = 4315

type B13b = Vec<(usize,u64)>;

fn parse13b (filename: &str) -> B13b {
        read_to_string(filename)
        .unwrap_or_else( |err| {println!("{:?}", err); err.to_string()})
        .lines()
        .map( |l| l.to_string() )
        .collect::<Vec<String>>()
        [1]
        .split(",")
        .map( |e| e.parse::<u64>().unwrap_or(1) )
        .enumerate()
        .filter( |(_i,e)| 1 < *e)
        .collect::<B13b>()
}

fn doit13b (data: &[(usize,u64)]) -> u64 {
    let mut busprod :u64 = 1;
    let mut time :u64 = 0;
    for p in data.iter() { // p.0 = pus index  p.1 = bus number
        while ((time + p.0 as u64) % p.1) != 0 { time += busprod; }  // Increment time until current bus "arrives"
        busprod *= p.1; // increment is product of previous bus numbers
    }
    time
} // 556100168221141

fn day13 () {
    ::std::println!("== {}:{} ::{}::day13() ====", std::file!(), core::line!(), core::module_path!());
    let data = parse13("data/input13.txt");
    println!("Result 13a: {:?}", doit13a(&data));
    let data = parse13b("data/input13.txt");
    println!("Result 13b: {:?}", &doit13b(&data));
}
// Day 13
////////////////////////////////////////////////////////////////////////////////
// Day 14

fn parse14a (filename: &str) -> u64 {
    let mut andmask :u64 = 0;
    let mut ormask :u64 = 0;
    let mut sum :HashMap<u64, u64> = HashMap::new();
    Regex::new(r"(mask|mem\[(\d+)\]) = (.*)").unwrap()
    .captures_iter(&::std::fs::read_to_string(filename).unwrap())
    .map( |cap| 
        if cap.get(2).is_none() {
            andmask = u64::from_str_radix(&cap[3].replace("X", "1"), 2).unwrap();   // and mask
            ormask = u64::from_str_radix(&cap[3].replace("X", "0"), 2).unwrap();  // or mask
            (0, 0)
        } else {
          let addr = cap[2].parse::<u64>().unwrap();
          let val =  cap[3].parse::<u64>().unwrap();
          let newval = (val | ormask) & andmask;
          sum.insert(addr, newval);
          (0, 0)
        }
    ).count();
    sum.iter().fold( 0, |r, (_k,v)| r + v)
} // 17481577045893

fn addresses (pat: &str) -> Vec<u64> {
    let nbits = pat.matches('X').count() as u32;
    (0 .. (2 as u64).pow(nbits)).map( |n| {
        let mut ns = pat.to_string().replace("1", "0");
        for b in 0 .. nbits {
             ns = ns.replacen("X", if n & (1<<b) != 0 { &"1" } else { &"0" }, 1);
        }
        let ret = u64::from_str_radix(&ns, 2).unwrap();
        ret
    }).collect::<Vec<u64>>()
}

fn parse14b (filename: &str) -> u64 {
    let mut xmask :String = "".to_string();
    let mut ormask :u64 = 0;
    let mut andmask :u64 = 0;
    let mut sum :HashMap<u64, u64> = HashMap::new();
    Regex::new(r"(mask|mem\[(\d+)\]) = (.*)").unwrap()
    .captures_iter(&::std::fs::read_to_string(filename).unwrap())
    .map( |cap| 
        if cap.get(2).is_none() {
            xmask = cap[3].to_string();   // X mask
            ormask = u64::from_str_radix(&cap[3].replace("X", "0"), 2).unwrap();  // or mask
            andmask = u64::from_str_radix(&cap[3].replace("0", "1").replace("X", "0"), 2).unwrap();
            (0, 0)
        } else {
          let addy = cap[2].parse::<u64>().unwrap();
          let val =  cap[3].parse::<u64>().unwrap();
          for mask in addresses(&xmask) {
              let addr2 = ((addy | ormask) & andmask) | mask;
             sum.insert( addr2, val);
          }
          (0, 0)
        }
    ).count();
    sum.iter().fold( 0, |r, (_k,v)| r + v)
} // 4160009892257

fn day14 () {
    ::std::println!("== {}:{} ::{}::day14() ====", std::file!(), core::line!(), core::module_path!());
    let data = parse14a("data/input14.txt");
    println!("Result 14a: {:?}", data);
    let data = parse14b("data/input14.txt");
    println!("Result 14b: {:?}", data);
}
// Day 14
////////////////////////////////////////////////////////////////////////////////
// Day 15

fn doit15a (data: &[usize], loopmax: usize) -> (usize, usize) {
    let mut h: HashMap<usize, usize> // <num, lastIdx>
        = HashMap::new();
    let mut last = (data[0], 0);

    for i in 1..data.len() {
         h.insert(last.0, last.1);
         last=(data[i], i);
    }
    for i in data.len()..loopmax {
        let lst = last;
        if let Some(prev) = h.get(&last.0) { // Seen before, so new lastnum is difference in distance
            last = (last.1-prev, i);
        } else {// First time lastnum seen, so put 0
            last = (0, i);
        }
        h.insert(lst.0, lst.1);
    }
    last
} // 763 1876406

fn day15 () {
    let mut d = util::delta();
    ::std::println!("== {}:{} ::{}::day15() ====", std::file!(), core::line!(), core::module_path!());
    println!("Result 15a: {:?}", doit15a(&[0,14,1,3,7,9], 2_0_20));
    d();
    println!("Result 15b: {:?}", doit15a(&[0,14,1,3,7,9], 30_000_000)); // 30000000
    d();
} // 763 1876406
// Day 15
////////////////////////////////////////////////////////////////////////////////
// Day 16

fn parse16a (file: &str) -> HashMap::<usize, HashSet<String>> {
    let mut h = HashMap::new(); // New empty HashMap
    for line in file.lines() { // Over all field names and valid ranges lines...
      let cap = Regex::new(r"([^:]+): (.*)-(.*) (.*)-(.*)").unwrap().captures(line).unwrap();
      for j in cap[2].parse::<usize>().unwrap() ..= cap[3].parse::<usize>().unwrap() {
            h.entry(j).or_insert(HashSet::new()).insert(cap[1].to_string());
      }
      for j in cap[4].parse::<usize>().unwrap() ..= cap[5].parse::<usize>().unwrap() {
            h.entry(j).or_insert(HashSet::new()).insert(cap[1].to_string());
      }
    }
    //h.iter().inspect( |(i,e)| println!("\n\n{}\n{:?}", i, e) ).count();
    h
}

fn doit16a (h: &HashMap::<usize, HashSet<String>>, file: &str) -> usize {
    let mut v = vec!();
    for line in file.lines() {
        line.split(",").enumerate().for_each( |(_i,n)| {
            let num = n.parse::<usize>().unwrap();
            if h.get(&num).is_none() {
                v.push(num);
            }
        })
    }
    v.iter().sum()
} // 27802

fn doit16b (h: &HashMap::<usize, HashSet<String>>, file: &str) -> usize {
    let tickets =  // Read the vector of vector of numbers
    file.lines()
    .map( |line| line.split(",").map( |e| e.parse::<usize>().unwrap()).collect::<Vec<usize>>() ) // vec of ticket fields
    .filter( |v| v.iter().all( |n| h.get(&n).is_some() ) )
    .collect::<Vec<_>>();

    // Vector of tickets of hashmaps... each hash set contains possible field names
    let mut newtickets :Vec<Vec<HashSet<String>>> = vec!();
    for ticket in &tickets { // over all Vec<Vec<usize>>
        let mut newticket :Vec<HashSet<String>> = vec!(); // Create empty new Vec<HashSet<usize>>
        for num in ticket {  // for each Vec<usize>
            newticket.push(h.get(&num).unwrap().clone()); // push HashSet<usze> onto Vec<Hashset<usize>>
        }
        newtickets.push(newticket);
    }

    // Initial vector of hashsets to intersect with each ticket's hashmap...in
    // the end should have a vector of hashmaps continaing just one string (the
    // field naem for that field index)
    let mut finalfields :Vec<HashSet<String>> = vec!();
    for i in 0..20 {
        finalfields.push(newtickets[0][i].clone()) // push HashSet<usize> onto Vec<HashSet<usize>>
    }

    // Intersect all the columns into a vector of sets of possible field names.  The ones
    // with a single field name identify that field name.  Must continue to filter ...
    for f in 0..20 {
        for ticket in newtickets.iter() {
            finalfields[f].retain( |e| ticket[f].contains(e))
        }
    }

    let mut sum = 1;
    let myfields = vec![79,67,101,89,131,107,139,113,127,83,137,53,71,149,73,97,59,61,109,103];
    // Continuously identify the single field row, then remove it from all the others since it
    // is the only field that can be that field.
    loop {
        let single = finalfields.iter().find( |e| e.len() == 1 );
        if single.is_none() { break }
        let single = single.unwrap().iter().nth(0).unwrap().to_string();
        for idx in 0..20 {
            if finalfields[idx].len() == 1 && single.contains("departure") {
               sum *= myfields[idx];
            }
            finalfields[idx].retain( |e| *e != single);
        }
    }
    sum
} // 279139880759


fn day16 () {
    ::std::println!("== {}:{} ::{}::day16() ====", std::file!(), core::line!(), core::module_path!());
    let file = read_to_string("data/input16a.txt").unwrap();
    let data = parse16a(&file);
    let file = read_to_string("data/input16b.txt").unwrap();
    println!("Result 16a: {:?}", doit16a(&data, &file));
    println!("Result 16b: {:?}", doit16b(&data, &file));
}
// Day 16
////////////////////////////////////////////////////////////////////////////////
// Day 17
type B17 = HashMap<(i32, i32, i32), usize>; // Let's do {location: [neighbord count|alive] }

fn parse17 (file: &str) -> B17 {
    let mut hs = B17::new();
    for (y,l) in file.lines().enumerate() {
        for (x,c) in l.chars().enumerate() {
            if c == '#' { hs.insert((x as i32,y as i32,0i32), 1); }
        }

    }
    hs
}

// Increment all my neighbors' counts
fn neighborsInc (s:&mut B17, (m,n,o):&(i32,i32,i32)) {
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                if x==0 && y==0 && z== 0 { continue }
                // Might have to create a new empty spot
                *s.entry((m+x,n+y,o+z)).or_insert(0) += 2; // Increment count for alive or dead spot
            }
        }
    }
}

fn doit17a (s: &mut B17) -> B17 {
    let mut snew = B17::new();
    let mut locs :Vec<(i32,i32,i32)> = vec!();
    for (l,_v) in s.iter() { // Consider every alive location
        locs.push((l.0, l.1, l.2)); 
    }
    for l in locs {         // And increment all my neighbors
        neighborsInc(s, &l);
    }

    for (l,v) in s { // Consider every touched location
         // Maybe carry life over
         if (3 == *v>>1) || ((1 == (*v & 1)) && (2 == *v>>1)) {
            snew.insert(*l, 1);
         } 
    }
    snew
}

type B17b = HashMap<(i32, i32, i32, i32), usize>; // Let's do {location: [neighbord count|alive] }

fn parse17b (file: &str) -> B17b {
    let mut hs = B17b::new();
    for (y,l) in file.lines().enumerate() {
        for (x,c) in l.chars().enumerate() {
            if c == '#' { hs.insert((x as i32,y as i32, 0i32, 0i32), 1); }
        }

    }
    hs
}
fn neighborsInc17b (s:&mut B17b, (m,n,o,p):&(i32,i32,i32,i32)) {
    for w in -1..=1 {
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                if x==0 && y==0 && z== 0 && w==0{ continue }
                // Might have to create a new empty spot
                *s.entry((m+x,n+y,o+z,p+w)).or_insert(0) += 2; // Increment count for alive or dead spot
            }
        }
    }
    }
}
fn doit17b (s: &mut B17b) -> B17b {
    let mut snew = B17b::new();
    let mut locs :Vec<(i32,i32,i32,i32)> = vec!();
    for (l,_v) in s.iter() { // Consider every alive location
        locs.push((l.0, l.1, l.2, l.3)); 
    }
    for l in locs {         // And increment all my neighbors
        neighborsInc17b(s, &l);
    }

    for (l,v) in s { // Consider every touched location
         // Maybe carry life over
         if (6 == ((*v) & 254)) || (5 == ((*v) & 255)) {
         //if (3 == *v>>1) || ((1 == (*v & 1)) && (2 == *v>>1)) {
            snew.insert(*l, 1);
         } 
    }
    snew
}

fn day17 () {
    ::std::println!("== {}:{} ::{}::day17() ====", std::file!(), core::line!(), core::module_path!());
    let file = read_to_string("data/input17.txt").unwrap();
    let mut s = parse17(&file);
    for _ in 1..=6 { s = doit17a(&mut s); }
    println!("Result 17a: {:?}", s.len()); // 375

    let file = read_to_string("data/input17.txt").unwrap();
    let mut s = parse17b(&file);
    //println!("{} |{}|", 0, s.len());
    for _ in 1..=6 {
         s = doit17b(&mut s);
         //println!("{} |{}|", l, s.len());
    }
    println!("Result 17b: {:?}", s.len()); // 2192
}
// Day 17
////////////////////////////////////////////////////////////////////////////////
// Day 18

fn solve18a (mut v: &[char]) -> (usize, &[char]) {
    let mut nums = vec!();
    let mut ops  = vec!();
    loop {
        //println!("{:?}  \x1b[33m{:?}\x1b[0m  {:?}", v, nums, ops);
        let c = v[0];
        v = &v[1..];
        match c {
            '*'|'+' => ops.push(c),
            '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9'|'(' =>  {
                let mut num :usize;
                if c == '(' {
                    let ret = solve18a(v);
                    num = ret.0;
                    v = ret.1;
                } else {
                    num = c.to_string().parse::<usize>().unwrap();
                }
                match ops.pop() {
                    Some('*') => { num *= nums.pop().unwrap(); },
                    Some('+') => { num += nums.pop().unwrap(); },
                    _ => ()
                }
                nums.push(num);
            },
            _ => ()
        }
        if c==')' || v.is_empty() { break }
    } // loop
    (nums.pop().unwrap(), v)
} // 510009915468

fn solve18b (mut v: &[char]) -> (usize, &[char]) {
    let mut nums = vec!();
    let mut ops  = vec!();
    loop {
        //println!("{:?}  \x1b[33m{:?}\x1b[0m  {:?}", v, nums, ops);
        let c = v[0];
        v = &v[1..];
        match c {
            '*'|'+' => ops.push(c),
            '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9'|'(' =>  {
                let mut num :usize;
                if c == '(' {
                    let ret = solve18b(v);
                    num = ret.0;
                    v = ret.1;
                } else {
                    num = c.to_string().parse::<usize>().unwrap();
                }
                if let Some('+') = ops.last() {
                    ops.pop(); num += nums.pop().unwrap();
                }
                nums.push(num);
            },
            _ => ()
        }
        if c==')' || v.is_empty() { break }
    } // loop
    let mut num = nums.pop().unwrap();
    while let Some('*') = ops.last() {
        ops.pop();
        num *= nums.pop().unwrap();
    }
    (num, v)
} // 321176691637769

type B18 = Vec<Vec<char>>;
fn parse18 (file: &str) -> B18 {
    file
    .lines()
    .map( |line| line.chars().collect::<Vec<char>>())
    .collect::<B18>()
}

fn doit18a (data: &B18) -> usize { data.iter().map( |v| solve18a(&v).0 ).sum() }
fn doit18b (data: &B18) -> usize { data.iter().map( |v| solve18b(&v).0 ).sum() }

fn day18 () {
    ::std::println!("== {}:{} ::{}::day18() ====", std::file!(), core::line!(), core::module_path!());
    let file = read_to_string("data/input18.txt").unwrap();
    let data = parse18(&file);
    println!("Result 18a: {:?} 510009915468", doit18a(&data));
    println!("Result 18b: {:?} 321176691637769", doit18b(&data));
}
// Day 18
////////////////////////////////////////////////////////////////////////////////
// Day 19

type B19a = HashMap<usize,Vec<Vec<usize>>>;
type SS = HashSet<String>;
type B19m = HashMap<usize,SS>;

fn parse19b (file: &str)  -> Vec<String> {
    file.lines().map(|l| l.to_string()).collect::<Vec<String>>()
}

fn parse19 (file: &str)  -> (B19a, B19m) {
    let mut data = B19a::new();
    let mut mem =  B19m::new();

    for t in
        file.lines()
        .map( |line| Regex::new(r"(\d+): (.*)") .unwrap() .captures(line).unwrap() )
        .map( |cap| (cap[1].to_string(),
                     cap[2].split(" | ").map(|e| e.to_string()).collect::<Vec<String>>()) ) {
        
        if t.1[0].chars().nth(0).unwrap() == '"' {
            let mut hs = HashSet::new();
            hs.insert(t.1[0].chars().nth(1).unwrap().to_string());
            mem.insert( t.0.parse::<usize>().unwrap() as usize , hs);  // Add the single letter to the cache for this ID
        } else {
            data.insert(
                t.0.parse::<usize>().unwrap(),
                t.1.iter().map( |s| s.split(" ")
                              .map( |n| n.parse::<usize>().unwrap())
                              .collect::<Vec<usize>>() )
                .collect::<Vec<Vec<usize>>>()
            );
        }
    }
    //for a in b19 { println!("{:?}", a); } println!("{:?}", s);
    (data, mem)
}

fn generateallstrings19(data: &mut B19a, mem: &mut B19m) -> SS {
    let mut found = -1;
    'top:
    loop {
        let keys = data.keys().clone();
        let mut hs :SS = HashSet::new(); // Add all possible generated string to this hash set
        'next:
        for num in keys {
            let vv :&Vec<Vec<usize>> = data.get(num).unwrap(); //  Look for a scanner entry we can resolve and move to  hash table

            // If not all numbers in this entry are in the cache, try next...
            if vv.iter().any( |v| v.iter().any( |n| mem.get(n).is_none())) {
                 continue 'next;
            }

            for v in vv { // Over vector of [num, ...] create set of cartesianed strings
                let mut chs :SS = HashSet::new();
                chs.insert("".to_string());
                for n in v { // chs crossproduct ss
                    if let Some(ss) = mem.get(n) { // might get from cache {str, ...}
                        let mut _chs :SS = HashSet::new(); // new chs
                        for a in chs {
                            for b in ss {
                                _chs.insert( a.clone() + b);
                            }
                        }
                        chs = _chs;
                    } else {
                        println!("ERROR: CAN'T FIND {} IN {:?} SHOULD NEVER HAPPEN", n, vv);
                    }
                }
                hs.extend(chs); // Add set of new strings to set
            }
            found = *num as i32; 
            break;
        }
        if found != -1 {
            data.remove(&(found as usize));
            mem.insert(found as usize, hs); // Add set of new strings to cache
            found = -1;
            continue 'top; // Success! Start from top again
        }
        break; // Go to the end without doing work so stop
    }

    // Peek at left over language:
    data.iter().for_each( |(k,v)| println!("leftover {:?} {:?}", k,v));

    // Combine hashmap of {String,...} to {String,...}
    let mut result :SS = HashSet::new();
    for (_,ss) in mem {
        for s in ss.iter() {
            result.insert(s.clone());
        }
    }
    result
}

fn generatetokens19(data: &mut B19a, mem: &mut B19m) {
    let mut found = -1;
    'top:
    loop {
        let keys = data.keys().clone();
        let mut hs :SS = HashSet::new(); // Add all possible generated string to this hash set
        'next:
        for num in keys {
            let vv :&Vec<Vec<usize>> = data.get(num).unwrap(); //  Look for a scanner entry we can resolve and move to  hash table

            // If not all numbers in this entry are in the cache, try next...
            if vv.iter().any( |v| v.iter().any( |n| mem.get(n).is_none())) {
                 continue 'next;
            }

            for v in vv { // Over vector of [num, ...] create set of cartesianed strings
                let mut chs :SS = HashSet::new();
                chs.insert("".to_string());
                for n in v { // chs crossproduct ss
                    if let Some(ss) = mem.get(n) { // might get from cache {str, ...}
                        let mut _chs :SS = HashSet::new(); // new chs
                        for a in chs {
                            for b in ss {
                                _chs.insert( a.clone() + b);
                            }
                        }
                        chs = _chs;
                    } else {
                        println!("ERROR: CAN'T FIND {} IN {:?} SHOULD NEVER HAPPEN", n, vv);
                    }
                }
                hs.extend(chs); // Add set of new strings to set
            }
            found = *num as i32; 
            break;
        }
        if found != -1 {
            data.remove(&(found as usize));
            mem.insert(found as usize, hs); // Add set of new strings to cache
            found = -1;
            continue 'top; // Success! Start from top again
        }
        break; // Go to the end without doing work so stop
    }
}

fn doit19a(tokens: &SS, sentences: &Vec<String>) -> usize {
    sentences.iter().filter( |s|
        if tokens.get(*s).is_some() { true } else { false }
    ).count()
}

//11: 42 31 | 42 11 31
fn parse11(tokens: &B19m, s: &str, prods: &[usize]) -> bool {
    return {
        let mut p = vec!(42, 31);
        p.extend_from_slice(prods);
        parse0(tokens, s, &p)
    } || {
        let mut p = vec!(42, 11, 31);
        p.extend_from_slice(prods);
        parse0(tokens, s, &p)
    }
}


//8: 42 | 42 8
fn parse8(tokens: &B19m, s: &str, prods: &[usize]) -> bool {
    return {
        let mut p = vec!(42);
        p.extend_from_slice(prods);
        parse0(tokens, s, &p)
    } || {
        let mut p = vec!(42, 8);
        p.extend_from_slice(prods);
        parse0(tokens, s, &p)
    }
}

fn parse0 (tokens: &B19m, s: &str, prods: &[usize]) -> bool {
    if 0==prods.len() && 0==s.len() { return true } // perfect match  GOOD
    if 0==prods.len() || 0==s.len() { return false; } // ran out of productions or sentence BAD
    match prods[0] { // recursive decent
        8 => return parse8(tokens,  s, &prods[1..]),
        11 => return parse11(tokens, s, &prods[1..]),
        tid => {
            for t in tokens[&tid].iter() {
                let l = t.len();
                if s.starts_with(t)  && parse0(tokens, &s[l..], &prods[1..]) {
                    return true
                }
            }
        }
    }
    return false
}

//0: 8 11  <- try and apply this production
fn doit19b (tokens: &B19m, sentences: &Vec<String>) -> usize {
    sentences.iter().filter( |s|
        parse0(&tokens, s, &[8, 11])
    ).count()
}

fn day19 () {
    ::std::println!("== {}:{} ::{}::day19() ====", std::file!(), core::line!(), core::module_path!());

    // Consider all possible tokens that can be generated and compare sentences against them.
    let filea = read_to_string("data/input19a.txt").unwrap();
    let (mut data, mut mem) = parse19(&filea);
    let tokens = generateallstrings19(&mut data, &mut mem);
    let fileb = read_to_string("data/input19b.txt").unwrap();
    let sentences = parse19b(&fileb);
    println!("Result 19a: {:?}", doit19a(&tokens, &sentences));

    // This time use the tokenizer table for a recursive decent parser.
    let fileab = read_to_string("data/input19ab.txt").unwrap();
    let (mut datab, mut memb) = parse19(&fileab);
    generatetokens19(&mut datab, &mut memb);
    println!("Result 19b: {:?}", doit19b(&memb, &sentences));

}
// Day 19
////////////////////////////////////////////////////////////////////////////////
// Day 20
#[derive(Debug)]
struct Piece {
    id: usize,
    vals: Vec<[usize;4]>
}

impl Piece {
    fn new (id: usize, mut nums: Vec<[usize;2]>) -> Piece {
        let mut piece = Piece{id:id, vals:vec!()};
        piece.vals.push( [nums[0][0], nums[1][0], nums[2][1], nums[3][1]]);
        piece.vals.push( [nums[1][0], nums[2][0], nums[3][1], nums[0][1]]);
        piece.vals.push( [nums[2][0], nums[3][0], nums[0][1], nums[1][1]]);
        piece.vals.push( [nums[3][0], nums[0][0], nums[1][1], nums[2][1]]);
        nums[0][0] ^= nums[0][1]; // Reverse top
        nums[0][1] ^= nums[0][0];
        nums[0][0] ^= nums[0][1];
        nums[2][0] ^= nums[2][1]; // Reverse bottom
        nums[2][1] ^= nums[2][0];
        nums[2][0] ^= nums[2][1];
        let e = nums[1]; // Swap left/right side
        nums[1] = nums[3];
        nums[3] = e;
        piece.vals.push([nums[0][0], nums[1][0], nums[2][1], nums[3][1]]);
        piece.vals.push([nums[1][0], nums[2][0], nums[3][1], nums[0][1]]);
        piece.vals.push([nums[2][0], nums[3][0], nums[0][1], nums[1][1]]);
        piece.vals.push([nums[3][0], nums[0][0], nums[1][1], nums[2][1]]);
        piece
    } // new
}

type B20 = HashMap<usize,Piece>;

fn bin2nums (s: &str) -> [usize;2] {
    // The number, and the number with 10 bits reversed
    [usize::from_str_radix(s, 2).unwrap(),
     usize::from_str_radix(&s.chars().rev().collect::<String>(), 2).unwrap()]
}

fn parse20 (file: &str) -> B20 {
    let mut itr = file.lines();
    let mut hm = B20::new();
    let mut count = 0;
    while let Some(line) = itr.next() {
        let id = &Regex::new(r"Tile (\d+):").unwrap().captures(line).unwrap()[1].parse::<usize>().unwrap();
        let mut v = vec!();
        for _ in 0..10 { v.push(itr.next().unwrap().replace("#", "1").replace(".", "0")); }
        let piece =
        Piece::new(*id, vec!(
            bin2nums(&v[0]), // top
            bin2nums(&v.iter().map(|s|s.chars().nth(9).unwrap()).collect::<String>()), // right
            bin2nums(&v[9].chars().rev().collect::<String>()), // bott
            bin2nums(&v.iter().map(|s|s.chars().nth(0).unwrap()).rev().collect::<String>()), // left
        ));
        hm.insert(count, piece);
        count += 1;
        itr.next(); // skip blankline
    }
    hm
}
type B20a = HashMap<usize, (usize, usize)>;

fn tryall (_pieces: &B20, table: &B20a, keys: &HashSet<usize> ) {
    table.iter().for_each( |l| { println!("hm {:?}", l) } );
    keys.iter().for_each( |l| { println!("keys {:?}", l) } );
}

fn doit20a (pieces: &B20) -> usize {
    let mut table = B20a::new(); // loc -> piece, rotation // Table
    let mut keys = pieces.keys().map(|e|*e).collect::<HashSet<usize>>(); // Pieces to place
    // Place 0
    keys.remove(&0);
    table.insert(0, (0, 0)); // loc -> piece index, rotation

    // Any fits?
    tryall(pieces, &table, &keys);

    //data.iter() .inspect( |l| { println!("<< {:?}", l) } ) .counat()
    0
}

fn day20 () {
    ::std::println!("== {}:{} ::{}::day20() ====", std::file!(), core::line!(), core::module_path!());
    let file = read_to_string("data/input20.txt").unwrap();
    let data = parse20(&file);
    println!("Result 20a: {:?}", doit20a(&data));
    //println!("Result 20b: {:?}", doit20b(&data));
}
// Day 20
////////////////////////////////////////////////////////////////////////////////
// Day 99
type B99 = HashMap<usize,String>;

fn parse99 (file: &str) -> B99 {
    Regex::new(r"(\w?) (.*)").unwrap().captures_iter( &file  )
    .inspect( |l| println!("| {:?}", l) )
    .filter( |cap| &cap[1] != "" ) // all newlines become an empty capture for some reason
    .map( |cap| {
        let line = cap[1].to_string();
        line
    })
    .enumerate()
    .collect::<B99>()
}

fn doit99a (data: &B99) -> usize {
    data.iter()
    .inspect( |l| { println!("<< {:?}", l) } )
    .count()
}

fn day99 () {
    ::std::println!("== {}:{} ::{}::day99() ====", std::file!(), core::line!(), core::module_path!());
    let file = read_to_string("data/input99.txt").unwrap();
    let data = parse99(&file);
    println!("Result 99a: {:?}", doit99a(&data));
    //println!("Result 99b: {:?}", doit99b(&data));
}
// Day 99
////////////////////////////////////////////////////////////////////////////////
// Day 98

type B98 = HashMap<usize,String>;

fn parse98 (file: &str)  -> B98{
    //let mut st = Strings::new();

    file
    .lines()
    .map( |line|
         Regex::new(r"\n?([^:]+): (\d+)-(\d+) or (\d+)-(\d+)").unwrap().captures(line)
    ).filter_map( |e| e )
    .map( move |cap| (1 .. cap.len()).map( |i| cap[i].to_string() ).collect::<Vec<String>>() )
    .inspect(db)
    //.filter( |cap| &cap[1] != "" ) // all newlines become an empty capture for some reason
    //.map( |cap| {
    //    let line = cap[0].to_string();
    //    line
    //})
    .count();
    B98::new()
}

fn doit98a (data: &B98) -> usize {
    data.iter()
    .inspect( db )
    .count()
}

fn day98 () {
    ::std::println!("== {}:{} ::{}::day98() ====", std::file!(), core::line!(), core::module_path!());
    let s = Strings::new();
    let file = read_to_string("data/input98.txt").unwrap();
    let data = parse98(&file);
    println!("Result 98a: {:?} {:?}", s, doit98a(&data));
    //println!("Result 98b: {:?}", doit98b(&data));
}
// Day 98
////////////////////////////////////////////////////////////////////////////////
// Main

pub fn main() {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    if false {
    day1();
    // 892 + 1128 = 2020, multiplied = 1006176
    // 1128 + 892 = 2020, multiplied = 1006176
    // 890 + 874 + 256 = 2020, multiplied = 199132160
    // 890 + 256 + 874 = 2020, multiplied = 199132160
    // 874 + 890 + 256 = 2020, multiplied = 199132160
    // 874 + 256 + 890 = 2020, multiplied = 199132160
    // 256 + 890 + 874 = 2020, multiplied = 199132160
    // 256 + 874 + 890 = 2020, multiplied = 199132160
    day2();
    // Valid passwords: 483
    // Valid passwords 2nd approach: 482
    day3();
    // First stage result 286
    // Second stage result 3638606400
    day4();
    // Valid passports v1 242
    // Valid passports v2 186
    day5();
    // Your seat number: 524
    day6();
    // Result A: Ok(6612)
    // Result B: Ok(3268)
    day7();
    // Result A: 144
    // Result B: 5956
    day8();
    // Result 8a: Some(1867)
    // Result 8b: Some(1303)
    day9();
    // Result 9a: 18272118
    // Result 9b: 2186361
    day10();
    // Result 10a: res = 36 69 product = 2484
    // Result 10b: 15790581481472
    day11();
    // Result 11a: 2113
    // Result 11b: 1865
    day12();
    // Parse 12a: 1133
    // Parse 12b: 61053
    day13();
    // Result 13a: 4315
    // Result 13b: 556100168221141
    day14();
    // 17481577045893
    // 4160009892257
    day15();
    // Result 15a: 763
    // Result 15a: 1876406
    day16();
    // 27802
    // 279139880759
    day17();
    // Result 17a: 375
    // Result 17b: 2192
    day18();
    // Result 18a: 510009915468
    // Result 18b: 321176691637769
    day19();
    // Result 19a: 129
    // Result 19b: 243
    day99();
    day98();
    }
    day20();

}

// Main
////////////////////////////////////////////////////////////////////////////////