
pub fn main () {
    println!("{:?}", std::env::args());
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    ::hyphen::main();
}
