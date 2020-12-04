use std::collections::HashMap;
use regex::Regex;

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
    match ::std::fs::read_to_string("data/input.day1") {
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
    Regex::new(r"(\d+)-(\d+) (.): (.*)")
    .unwrap()
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
    println!("Valid passwords: {}", validate_passwords("data/input.day2"));
    println!("Valid passwords 2nd approach: {}", validate_passwords2("data/input.day2"));
}

// Day 2
////////////////////////////////////////////////////////////////////////////////
// Day 3

fn walk ( filename: &str, skiptable: &[(usize, usize)]) -> usize {
    ::std::fs::read_to_string(filename).unwrap().lines()
    .fold(
        (0,1,0,0),
        |(idx, rowskip, col, result), row| {
        if rowskip == 1 {
            (idx+1,
             skiptable[idx % skiptable.len()].1,
             col + skiptable[idx % skiptable.len()].0,
             result + (0 < row.len() &&
                      '#' == row.chars().nth(col % row.len()).unwrap() ) as usize )
        } else {
           (idx, rowskip-1, col, result)
        }
    }
    ).3
}

fn day3 () {
    ::std::println!("== {}:{} ::{}::day3() ====", std::file!(), core::line!(), core::module_path!());
    println!("First stage result {}", walk("data/input3.txt", &[(3,1)]));
    println!("Second stage result {:?}",
        [(1,1),(3,1),(5,1),(7,1),(1,2)]
        .iter()
        .map( |p| walk("data/input3.txt", &[*p]) )
        .product::<usize>()
    );
}

// Day 3
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
    }
    day3();
    // First stage result 286
    // Second stage result 3638606400
}

// Main
////////////////////////////////////////////////////////////////////////////////