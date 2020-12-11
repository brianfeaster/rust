use std::collections::{HashMap, HashSet};
use regex::Regex;
use std::fs::{read_to_string};

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

fn parse8 (thefile: &str) -> Prog {
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
    let mut prog = parse8(thefile);
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
    let mut prog = parse8(thefile);
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

fn parse11 (filename: &str) -> B11 {
    read_to_string(filename).unwrap().lines().enumerate()
    .map(
        |(y, l)|
        l.chars().enumerate()
        .map(|(x,c)| ((x as i32 ,y as i32 ), match c { '.'=>1, 'L'=>2, '#'=>3, _=>0 } as i32))
        .collect::<Vec<_>>() )
    .flatten()
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

fn day11 () {
    ::std::println!("== {}:{} ::{}::day11() ====", std::file!(), core::line!(), core::module_path!());
    let h :B11 = parse11("data/input11.txt");
    println!("Result 11a: {:?}", day11a(&h)); // 2113
    println!("Result 11b: {:?}", day11b(&h)); // 1865
}
// Day11 
////////////////////////////////////////////////////////////////////////////////
// Day j
type Adtj = HashMap<usize,String>;

fn parsej (filename: &str) -> Adtj {
    read_to_string(filename).unwrap()
    .lines()
    .enumerate()
    .map( |(i,e)| {
         println!("<< {}", e);
         (i,e.to_string())
    })
    .collect::<Adtj>()
}

fn doitja (data: &Adtj) -> usize {
    data
    .values()
    .enumerate()
    .map( |(i,e)| (e, data.values().take(i+1).collect::<Vec<_>>()))
    .inspect( |e| println!("{:?}", e))
    .count()
}

fn dayj () {
    ::std::println!("== {}:{} ::{}::dayj() ====", std::file!(), core::line!(), core::module_path!());
    let data = parsej("data/inputj.txt");
    println!("Result ja: {:?}", doitja(&data));
    //println!("Result Jb: {:?}", parsej("data/inputj.txt"));
}
// Day j
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
    }
    dayj();
}

// Main
////////////////////////////////////////////////////////////////////////////////