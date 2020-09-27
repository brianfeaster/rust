mod fun;
mod util;
mod term;

fn main () {
    ::main::mainAsteroid();
    //::main::callFun(); // via lib.rs
    //crate::fun::main();
    //::main::mainGravity();
    //println!("{:?}", mainJson());
    //println!("!!! {:?}", mainJsonSerdes());
    //println!("map {:?}", ('🐘' .. '🐷').map(|x| (|x| x)(x)).collect::<Vec<char>>()); // type std::ops::RangeInclusive
}