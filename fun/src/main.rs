#![allow(non_snake_case)]
//mod fun;
//mod util;
//mod term;


fn main () {
    ::fun::mainAsteroid(); // ??? Is there a symbol to explicitly reference the root module or is "crate" and other modules the only symbols?
    //crate::fun::mainAsteroid(); // Can only reference from "crate::"" if you "mod ::fun"

    //::fun::callFun(); // via lib.rs which needs to "mod fun"

    //crate::fun::main();
    //::main::mainGravity();
    //println!("{:?}", mainJson());
    //println!("!!! {:?}", mainJsonSerdes());
    //println!("map {:?}", ('🐘' .. '🐷').map(|x| (|x| x)(x)).collect::<Vec<char>>()); // type std::ops::RangeInclusive
}