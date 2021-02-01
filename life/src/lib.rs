use ::std::sync::{Mutex};
use ::piston::*;
use ::graphics::{Graphics, DrawState, Viewport};
use ::opengl_graphics::{GlGraphics, OpenGL}; //Colored, Textured
use ::glutin_window::{GlutinWindow};

use ::utils::{Watch};
pub mod dbuff;
pub mod lifeh;
pub use lifeh::*;
pub use dbuff::*;

pub fn piston_draw_2d_gl (
    mdbuff: &Mutex<Dbuff>,
    whs: &(usize, usize, usize),
    deltas: &mut usize,
    ds: &DrawState,
    gfx: &mut GlGraphics,
) {
    let dbuff = mdbuff.lock().unwrap();
    let (ba, bb) = dbuff.buffs();
    let mut col :f32 = 0.0;
    let mut row :f32 = 0.0;

    // Scale to NDC "Normalized device coordinate" (still need to translate -1,-1)
    let sx = 2.0 / whs.0 as f32;
    let sy = 2.0 / whs.1 as f32;
    //let blue = [rf32(), rf32(), rf32(), 0.5];
    let black = [0.0, 0.0, 0.0, 1.0];
    //let darkblue =  [0.0, 0.0, 0.5, 1.0];
    let blue =  [0.0, 0.0, 1.0, 1.0];
    //let white = [1.0, 1.0, 1.0, 1.0];
    //let grey =  [0.4, 0.4, 0.4, 1.0];
    //let cyan =  [0.0, 5.0, 5.0, 1.0];
    //let green = [0.0, 1.0, 0.0, 1.0];
    //let red =   [1.0, 0.0, 0.0, 1.0];
    for i in 0..whs.2 {
        if ba[i] != bb[i] { // Compare buffer A with Buffer B for change in life state
            *deltas += 1;
            // Split the GoL square into two triangles
            let fx = col * sx - 1.0;
            let fy = row * sy - 1.0;
            let gx = fx + sx;
            let gy = fy + sy;
            if 0 < ba[i] {
                gfx.tri_list( ds, &blue, |f| f(&[[fx,fy], [gx,fy], [gx,gy], [fx,fy], [gx,gy], [fx,gy]]) );
                /*
            } else if -1 == ba[i] {
                gfx.tri_list( ds, &blue, |f| f(&[[fx,fy], [gx,fy], [gx,gy], [fx,fy], [gx,gy], [fx,gy]]) );
            } else if 2 == ba[i] {
                gfx.tri_list( ds, &white, |f| f(&[[fx,fy], [gx,fy], [gx,gy], [fx,fy], [gx,gy], [fx,gy]]) );
            } else if -2 == ba[i] {
                gfx.tri_list( ds, &grey, |f| f(&[[fx,fy], [gx,fy], [gx,gy], [fx,fy], [gx,gy], [fx,gy]]) );
                */
            } else {
                gfx.tri_list( ds, &black, |f| f(&[[fx,fy], [gx,fy], [gx,gy], [fx,fy], [gx,gy], [fx,gy]]) );
            }
        }
        col += 1.0;
        if col == whs.0 as f32 { col = 0.0; row += 1.0; }
    }

    // Slowly erase screen.  Disable recBlack plots for cool effect.
    if false { gfx.tri_list(ds, &[-1.0, -1.0, -1.0, 0.04], |f| f(&maketriangle()[..])); }
} // fn piston_draw_2d_callback

pub fn piston_draw_2d_gl_hash (
    arena: &ArenaBase,
    whs: &(usize, usize, usize),
    deltas: &mut usize,
    ds: &DrawState,
    gfx: &mut GlGraphics,
) {
    let black = [0.0, 0.0, 0.0, 1.0];
    let blue =  [0.0, 0.0, 1.0, 1.0];
    gfx.clear_color(black);
    // Scale to NDC "Normalized device coordinate" (still need to translate -1,-1)
    let sx = 2.0 / whs.0 as f32;
    let sy = 2.0 / whs.1 as f32;
    for ((x,y),_) in arena {
        *deltas += 1;
        let col = *x as f32;
        let row = *y as f32;
        // Split the GoL square into two triangles
        let fx = col * sx - 1.0;
        let fy = row * sy - 1.0;
        let gx = fx + sx;
        let gy = fy + sy;
        gfx.tri_list( ds, &blue, |f| f(&[[fx,fy], [gx,fy], [gx,gy], [fx,fy], [gx,gy], [fx,gy]]) );
    }
} // fn piston_draw_2d_gl_hash

fn maketriangle () -> [[f32;2];6] {
    [ [-1.0,-1.0], [1.0,-1.0], [1.0,1.0],
      [-1.0,-1.0], [1.0,1.0],  [-1.0,1.0] ]
}

////////////////////////////////////////////////////////////////////////////////

fn main_life_2d (w: usize, h: usize, cellsize: usize) -> bool {
    let mut life :Life = Life::new(w, h);
    //life.arena = None;
    let mut deltas :usize = 0;
    let winsize = (life.whs.0*cellsize, life.whs.1*cellsize);
    let mut watch = Watch::new();
    let mut s = 1;
    let ver = OpenGL::V3_2;
    let mut pwin =
        GlutinWindow::new(
            &WindowSettings::new( LIFE_TITLE, [winsize.0 as u32, winsize.1 as u32] )
                .graphics_api(ver)
                .exit_on_esc(true)
                .size(piston_window::Size{width:  winsize.0 as f64, height: winsize.1 as f64})
                .decorated(true)
        ).unwrap();
    let mut events = Events::new( EventSettings::new().max_fps(1111) );
    let mut glgfx = GlGraphics::new(ver);

    // Use this static viewport for rendering so window resizing doesn't affect normalized device coordinates.
    let viewport = Viewport {
        rect: [0 ,0, (winsize.0 * 2) as i32, (winsize.1 * 2) as i32],
        draw_size: [(winsize.0 * 2) as u32, (winsize.1 * 2) as u32],
        window_size: [winsize.0 as f64, winsize.1 as f64]};

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
                    Key::G      => { life.add_glider(0,0); },
                    Key::Q |
                    Key::Escape => { pwin.set_should_close(true); },
                    _ => ()
                }
            }
        },
        Event::Loop(Loop::Render(_args)) => {
            let whs = life.whs;
            //let mdbuff = &life.dbuffs.0;

            deltas = 0;
            let ctx = glgfx.draw_begin(viewport); // args.viewport()
                if life.arena.is_none() {
                    let mdbuff = life.gen_next(); // Start next gen threads, get last generated buff
                    piston_draw_2d_gl(mdbuff, &whs, &mut deltas, &ctx.draw_state, &mut glgfx);
                } else {
                    life.gen_next(); // Start next gen threads, ignore last generated buff
                    piston_draw_2d_gl_hash(life.arena.as_ref().unwrap(), &whs, &mut deltas, &ctx.draw_state, &mut glgfx);
                }
            glgfx.draw_end();

            //if 1==watch.tick { life.add_glider(0, 0); }
            watch.tick();
        },
        Event::Loop(Loop::AfterRender(_args)) => {
            //utils::sleep(100);
        },
        _ => ()
        } // match

        if let Some(w) = watch.mark(1.7) {
            //life.add_glider(0, 0);
            println!("\x1b[0;35m{} [{:.2}]\tgens:{} ∆lifes:{} spins:{} s={}\x1b[40m",
                life.threads, w.fps, life.tick, deltas, 0, s);
        }
        if 50000 < life.tick { pwin.set_should_close(true); }
    } // while Some(event)
    true
}

fn main_life_ascii (w: usize, h: usize, loopcount: usize) {
    let mut watch = utils::Watch::new();
    let mut life :Life = Life::new(w, h);
    life.arena = None;
    life.dbuff_en = false;
    let term = ::term::Term::new();
    term.terminalraw();
    loop {
        if 0 < loopcount && life.tick == loopcount { break; }
        if 0 == life.tick % 100 { life.add_glider(0,0); }
        match &term.getc()[..] {
          "q" => break,
          "c" => { life.clear(); }
          "g" => { life.add_glider(0,0); }
          " " => { life.randomize(8); }
          _ => ()
        }
        let mdbuff = life.gen_next().lock().unwrap();
        let (ba, bb) = mdbuff.buffs();
        watch.tick().mark(1.0);
        print!("\x1b[H Game of Life -- ASCII edition {}\x1b[0K", watch.fps);
        bb.iter().zip(ba.iter()).enumerate().for_each( |(i, ba)| {
            if 0 == i % w { println!("") }
            match ba {
                (0,0) => print!(" "), // dead
                (0,1) => print!("o"), // born
                (1,0) => print!("."), // died
                (1,2) => print!("O"), // survived
                (2,2) => print!("@"), // ancient
                (2,0) => print!(","), // death
                _ => print!("\x1b[31m\x1b[0m?"), // unknown
            }
        } );
    }
    term.done();
}

pub fn main () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    if true { main_life_2d(200, 200, 4); }
    else { main_life_ascii(140, 24, 10000); }
}

/* TODO: message passing pipeline
     Verify a thread crashing rith a lock and subsequent threads receiving
     invalid locks can communicate the new state (machine)/
  
println!("{:?}", event);
*/