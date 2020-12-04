#![allow(non_snake_case, dead_code)]

// This binary crate gets the following root crates added thanks to Cargo
//  ::main  ::util  ::ipc  ::term

fn main () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    //::main::mainAsteroid(); // ??? Is there a symbol to explicitly reference the root module or is "crate" and other modules the only symbols?  A: There are only crates and they canonically start with :: and create is the crate representing the current crate.
    //::main::mainGravity();
    //::main::main(); // lib.rs via Cargo
    //::main::lag::main();
    //::adventofcode::main();
    ::main::fun::main(); // lib.rs 'pub mod fun' adds fun to 
}