use ::std::sync::{Mutex};
use ::piston::*;
use ::graphics::{Graphics, DrawState, Viewport};
use ::opengl_graphics::{GlGraphics, OpenGL}; //Colored, Textured
use ::glutin_window::{GlutinWindow};
// local
use ::util::{Watch};
pub mod dbuff;
pub mod life;
pub use crate::life::*;
pub use crate::dbuff::*;

pub fn piston_draw_2d_gl (
    mdbuff: &Mutex<Dbuff>,
    whs: &(usize, usize, usize),
    deltas: &mut usize,
    ds: &DrawState, // the global transform
    gfx: &mut GlGraphics,
) {
    let dbuff = mdbuff.lock().unwrap();
    let (ba, bb) = dbuff.buffs();
    let mut col :f32 = 0.0;
    let mut row :f32 = 0.0;
    //gfx.clear_color([0.0, 0.0, 0.0, 1.0]);
    //glgfx.clear_stencil(1);

    //clear([0.0, 0.0, 0.0, 1.0], graphics);
    //let ds = DrawState { blend: None, stencil: None, scissor: None };
    //let ds = DrawState { blend: Some(draw_state::Blend::Alpha), stencil: None, scissor: None };

    // Scale to NDC "Normalized device coordinate" (still need to translate -1,-1)
    let sx = 2.0 / whs.0 as f32;
    let sy = 2.0 / whs.1 as f32;
    let black = [0.0, 0.0, 0.0, 1.0];
    //let blue = [rf32(), rf32(), rf32(), 0.5];
    let blue = [0.0, 0.0, 1.0, 1.0];
    let capacity = 6*17050;
    let mut polysblack = Vec::with_capacity(capacity);
    let mut polysblue = Vec::with_capacity(capacity);
    for i in 0..whs.2 {
        if ba[i] != bb[i] { // Compare buffer A with Buffer B for change in life state
            *deltas += 1;
            // Split the GoL square into two triangles
            let fx = col * sx - 1.0;
            let fy = row * sy - 1.0;
            let gx = fx + sx;
            let gy = fy + sy;
            if 0 != ba[i] {
                polysblue.push([fx,fy]);
                polysblue.push([gx,fy]);
                polysblue.push([gx,gy]);
                polysblue.push([fx,fy]);
                polysblue.push([gx,gy]);
                polysblue.push([fx,gy]);
            } else {
                polysblack.push([fx,fy]);
                polysblack.push([gx,fy]);
                polysblack.push([gx,gy]);
                polysblack.push([fx,fy]);
                polysblack.push([gx,gy]);
                polysblack.push([fx,gy]);
            }
        }
        if capacity == polysblack.len() { gfx.tri_list( ds, &black, |f| f(&polysblack) ); polysblack.clear(); }
        if capacity == polysblue.len()  { gfx.tri_list( ds, &blue,  |f| f(&polysblue) ); polysblue.clear(); }
        col += 1.0;
        if col == whs.0 as f32 { col = 0.0; row += 1.0; }
    }
    if 0 < polysblack.len() { gfx.tri_list( ds, &black, |f| f(&polysblack) ); }
    if 0 < polysblue.len()  { gfx.tri_list( ds, &blue, |f| f(&polysblue) ); }

    // Slowly erase screen.  Disable recBlack plots for cool effect.
    if false { gfx.tri_list(ds, &[-1.0, -1.0, -1.0, 0.06], |f| f(&maketriangle()[..])); }
} // fn piston_draw_2d_callback

fn maketriangle () -> [[f32;2];6] {
    [ [-1.0,-1.0], [1.0,-1.0], [1.0,1.0],
      [-1.0,-1.0], [1.0,1.0],  [-1.0,1.0] ]
}

/*
pub fn piston_draw_2d_callback (
    mdbuff: &Mutex<Dbuff>,
    whs: &(usize, usize, usize),
    deltas: &mut usize,
    graphics :&mut G2d,
) {
    let dbuff = mdbuff.lock().unwrap();
    let (ba, bb) = dbuff.buffs();
    let mut col :f32 = 0.0;
    let mut row :f32 = 0.0;
    //clear([0.0, 0.0, 0.0, 1.0], graphics);

    //let ds = DrawState { blend: None, stencil: None, scissor: None };
    let ds = DrawState { blend: Some(draw_state::Blend::Alpha), stencil: None, scissor: None };

    // Scale to NDC "Normalized device coordinate" (still need to translate -1,-1)
    let sx = 2.0 / whs.0 as f32;
    let sy = 2.0 / whs.1 as f32;
    let black = [0.0, 0.0, 0.0, 1.0];
    //let blue = [rf32(), rf32(), rf32(), 0.5];
    let blue = [0.0, 0.0, 1.0, 1.0];
    for i in 0..whs.2 {
        if ba[i] != bb[i] { // Compare buffer A with Buffer B for change in life state
            *deltas += 1;
            // Split the GoL square into two triangles
            let (fx, fy) = ( col * sx - 1.0, row * sy - 1.0 );
            let (gx, gy) = (fx + sx, fy + sy);
            let poly = [[fx,fy], [gx,fy], [gx,gy], [fx,fy], [gx,gy], [fx,gy]];
            if true || 0 != ba[i] {
                graphics.tri_list(&ds, &if 0!=ba[i]{blue}else{black}, |f| f(&poly));
            }
        }
        col += 1.0;
        if col == whs.0 as f32 { col = 0.0; row += 1.0; }
    }
    if false { // Slowly erase screen.  Disable recBlack plots for cool effect.
        graphics.tri_list(&ds, &[-1.0, -1.0, -1.0, 0.06],
             |f| f(&maketriangle()[..]));
    }
    //rectangle( [ 1.0, 0.0, 0.0, 1.0 ], [ 50.0, 50.0, 50.0, 50.0 ], context.transform, graphics);
} // fn piston_draw_2d_callback
*/

/*
pub fn piston_render (
    mdbuff: &Mutex<Dbuff>,
    whs: &(usize, usize, usize),
    pwin: &mut PistonWindow,
    event: Event,
) -> usize {
    let mut deltas :usize = 0; // Count number of changes from last aren
    pwin.draw_2d( &event,
        | _context:  piston_window::Context,
          graphics: &mut piston_window::G2d,
          _device:  &mut piston_window::GfxDevice
        | {
        piston_draw_2d_callback(mdbuff, whs, &mut deltas, graphics);
    });
    deltas
}
*/


////////////////////////////////////////////////////////////////////////////////

fn main_life_2d (w: usize, h: usize, cellsize: usize) -> bool {
    let mut life :Life = Life::new(w, h);
    let mut deltas :usize = 0;
    let winsize = (life.whs.0*cellsize, life.whs.1*cellsize);
    let mut watch = Watch::new();
    /*
    let mut pwin: PistonWindow =
        WindowSettings::new("ASCIIRhOIDS", [winsize.0 as u32, winsize.1 as u32])
            //.exit_on_esc(true)
            .size(piston_window::Size{width: winsize.0 as f64, height: winsize.1 as f64})
            .decorated(true)
            .build()
            .unwrap();
    pwin.set_max_fps(1111);
    */
    let mut s = 1;

    let ver = OpenGL::V3_2;
    let mut pwin =
        GlutinWindow::new(
            &WindowSettings::new( "ASCIIRhOIDS", [winsize.0 as u32, winsize.1 as u32] )
                .graphics_api(ver)
                .exit_on_esc(true)
                .size(piston_window::Size{width:  winsize.0 as f64, height: winsize.1 as f64})
                .decorated(true)
        ).unwrap();
    let mut events = Events::new( EventSettings::new().max_fps(1111) );
    let mut glgfx = GlGraphics::new(ver);

    let viewport = Viewport {
        rect: [0 ,0, (winsize.0 * 2) as i32, (winsize.1 * 2) as i32],
        draw_size: [(winsize.0 * 2) as u32, (winsize.1 * 2) as u32],
        window_size: [winsize.0 as f64, winsize.1 as f64]};

    /* Ghetto dump arena
    for y in 0..24 {
        println!("");
        let row = aa[y].lock().unwrap();
        for x in 0..80 {
                print!("{}", 1-row[x]);
        }
    }
    */


    //while let Some(event) = pwin.next() {
    while let Some(event) = events.next(&mut pwin) {
        match event { // events.next(&mut pwin)
        Event::Loop(Loop::Idle(IdleArgs{dt:_})) => { },
        Event::Input(Input::Resize(ResizeArgs{window_size:_, draw_size:_}), _) => { },
        Event::Input(Input::Button(ButtonArgs{state:st, button:Button::Keyboard(k), scancode:_}), _) => {
            if st == ButtonState::Press {
                match k {
                    Key::Space  => { s+=1; life.randomize(s);},
                    Key::C      => { life.clear(); },
                    Key::R      => { life.randomize(-1); },
                    Key::Q |
                    Key::Escape => { pwin.set_should_close(true); },
                    _ => ()
                }
            }
        },
        Event::Loop(Loop::Render(_args)) => {
            //println!("{:?}", d());
            let whs = life.whs;
            let mdbuff = life.gen_next(); // Start next gen threads, get last generated buff
            //deltas = piston_render(&mdbuff, &whs, &mut pwin, event);

            deltas = 0;
            let c = glgfx.draw_begin(viewport); // args.viewport()
            piston_draw_2d_gl(&mdbuff, &whs, &mut deltas, &c.draw_state, &mut glgfx);
            glgfx.draw_end();
            //util::sleep(1000);

            //if 1==watch.tick { life.add_glider(0, 0); }
            watch.tick();
        },
        _ => ()
        } // match

        if let Some(w) = watch.mark(2.0) {
            //life.add_glider(0, 0);
            println!("\x1b[0;35m{} [{:.2}]\tgens:{} ∆lifes:{} spins:{} s={}\x1b[40m",
                life.threads, w.fps, life.tick, deltas, 0, s);
        }
        if 50000 < life.tick { pwin.set_should_close(true); }
    } // while Some(event)
    true
}

fn main_life_ascii (w: usize, h: usize, loopcount: usize) {
    let mut watch = util::Watch::new();
    let mut life :Life = Life::new(w, h);
    life.dbuff_en = false;
    let term = ::term::Term::new();
    term.terminalraw();
    loop {
        if 0 < loopcount && life.tick == loopcount { break; }
        if 0 == life.tick % 100 { life.add_glider(0,0); }
        match &term.getc()[..] {
          "q" => break,
          "c" => { life.clear(); }
          " " => { life.randomize(8); }
          _ => ()
        }
        let mdbuff = life.gen_next().lock().unwrap();
        let (ba, _bb) = mdbuff.buffs();
        watch.tick().mark(1.0);
        print!("\x1b[H Game of Life -- ASCII edition {}\x1b[0K", watch.fps);
        ba.iter().enumerate().for_each( |(i, e)| {
            if 0 == i % w { println!("") }
            print!("{}", if *e == 0 { '.' } else { '@'});
        } );
    }
    term.done();
}

pub fn main () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    if true { main_life_2d(200, 200, 4); } else { main_life_ascii(140, 24, 10000); }
}

/* TODO: message passing pipeline
     Verify a thread crashing with a lock and subsequent threads receiving
     invalid locks can communicate the new state (machine)/
  
println!("{:?}", event);
200x200x4
6 [133.69]      gens:268 ∆lifes:1089 spins:0 s=1
6 [176.86]      gens:622 ∆lifes:1096 spins:0 s=1
6 [183.24]      gens:989 ∆lifes:546 spins:0 s=1
6 [190.31]      gens:1370 ∆lifes:224 spins:0 s=1
6 [203.13]      gens:1777 ∆lifes:85 spins:0 s=1
*/