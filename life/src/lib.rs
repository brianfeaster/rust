pub mod dbuff;
pub mod life;

pub use crate::life::*;
pub use crate::dbuff::*;

pub fn main () {
    println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    crate::life::main();
}