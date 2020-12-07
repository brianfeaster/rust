use std::collections::{HashMap, HashSet};
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
// Day j
fn ioerr () -> ::std::io::Error { ::std::io::Error::new(::std::io::ErrorKind::Other, "") }

fn doitj (filename: &str) -> ::std::io::Result<usize> {
    ::std::fs::read_to_string(filename)?
    .lines()
    .inspect( |e| println!("<< {}", e) )
    .count().checked_add(0).ok_or(ioerr())
}

fn dayj () {
    ::std::println!("== {}:{} ::{}::dayj() ====", std::file!(), core::line!(), core::module_path!());
    println!("Result A: {:?}", doitj("data/inputj.txt"));
    println!("Result B: {:?}", doitj("data/inputj.txt"));
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
    }
    dayj();
}

// Main
////////////////////////////////////////////////////////////////////////////////