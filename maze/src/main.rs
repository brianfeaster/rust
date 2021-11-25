use utils;

fn terminal_dump_map (hm: &::utils::HashMapDeterministic) {
    let (xmin, ymax) =
        hm.iter().fold(
            (std::i32::MAX, std::i32::MIN),
            | mut r, ((x,y),_) | {
                if *x  < r.0 { r.0 = *x };
                if r.1 < *y  { r.1 = *y };
                r
            }
        );
    let mut bot = 0;
    for ((x,y),k) in hm {
        let pos = ymax-y+1;
        if bot < pos { bot = pos }
        print!("\x1b[{};{}H\x1b[3{}m#\n", pos, x-xmin+1, k%8);
    }
    print!("\x1b[0m\x1b[{}H", bot);
    utils::flush();
    utils::sleep(5);
}

pub fn main () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    // Plotter callback
    let bplotter: Box::<dyn FnMut(&::utils::HashMapDeterministic)> =
        Box::new(move |hm| {
            print!("\x1b[2J");
            terminal_dump_map(hm);
        });

    // Start maze server  
    ::maze::start(6, 100, bplotter);
}