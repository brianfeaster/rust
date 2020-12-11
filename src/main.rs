// This binary crate gets the following root crates added thanks to Cargo:
//   ::main ::adventofcodde
// There are only crates.  They canonically start with :: but "create::"" is the crate's crate.

fn main () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    //::life::main();
    //::adventofcode::main();
    ::main::main();
}