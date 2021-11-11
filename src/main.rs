//#![allow(dead_code, unused_assignments, unused_imports, unused_variables, non_snake_case)]

// This binary crate gets the following root crates added thanks to Cargo:
//   ::main ::adventofcodde
// There are only crates.  They canonically start with :: but "create::"" is the crate's crate.

fn main () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    //::adventofcode::main();
    //::life::main();
    ::main::main();
    //::hyphen::main();
}