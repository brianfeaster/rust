
pub fn main () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    if true { ::pretty_env_logger::init() } // export RUST_LOG=debug
    ::log::error!("{:?}", ::hyphen::server());
}
