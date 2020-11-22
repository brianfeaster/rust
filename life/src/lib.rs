#![allow(dead_code, unused_assignments, unused_imports, unused_variables, non_snake_case)]
mod life;
mod dbuff;

/// Create random i32
pub fn ri32(m: i32) -> i32 { (::rand::random::<f32>() * m as f32) as i32 }

pub fn main () {
    ::pretty_env_logger::init();
    ::log::info!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    crate::life::main();
}