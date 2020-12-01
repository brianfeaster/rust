use std::collections::HashMap;


////////////////////////////////////////////////////////////////////////////////
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
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    match ::std::fs::read_to_string("data/input.txt") {
        Ok(filestr) => {
            let ht = numbers_to_hash(filestr);
            find2020(&ht);
            find2020x3(&ht);
        },
        e => println!("Unable to readfile {:?}", e)
    }
}
////////////////////////////////////////////////////////////////////////////////

pub fn main() {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());

    day1();
    // 892 + 1128 = 2020, multiplied = 1006176
    // 1128 + 892 = 2020, multiplied = 1006176
    // 890 + 874 + 256 = 2020, multiplied = 199132160
    // 890 + 256 + 874 = 2020, multiplied = 199132160
    // 874 + 890 + 256 = 2020, multiplied = 199132160
    // 874 + 256 + 890 = 2020, multiplied = 199132160
    // 256 + 890 + 874 = 2020, multiplied = 199132160
    // 256 + 874 + 890 = 2020, multiplied = 199132160
}