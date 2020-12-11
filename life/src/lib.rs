pub mod dbuff;
pub mod life;

pub use crate::life::{Life};
pub use crate::dbuff::{Dbuff};

pub fn main () {
    println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    crate::life::main();
}