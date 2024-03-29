#![allow(dead_code, unused_assignments, unused_imports, unused_variables, non_snake_case)]

// Bind the following modules to crate::
mod asciirhoids;
mod learn;
mod lag;
mod matrix;

//use crate::term::{Tbuff}; // Module path prefix "crate::"" and "self::"" are pedantic.
//use crate::lag::{Shape, Entity, Entities, EntityCast};

use ::std::{
    fs,
    thread::{spawn, JoinHandle},
    //collections::{HashMap},
    net::TcpListener,
    sync::mpsc::{channel, Receiver}};
use ::piston_window::*;

////////////////////////////////////////////////////////////////////////////////
// A fun console Asteroids game
//
pub fn mainAsteroid () {
    let (txipc, rxipc) = channel::<::ipc::Ipc>(); // 
    let doit :bool = true;
    match TcpListener::bind("127.0.0.1:7145") {
        Ok(l) => ::ipc::server(l, txipc),
        Err(_) => ::ipc::client(txipc),
    }
    if doit { 
        crate::asciirhoids::asciiteroids(rxipc); // Channel of Ipc objects
    } else {
        let mut ipc : ::ipc::Ipc = rxipc.recv().unwrap();
        loop {
            // receive
            loop {
                match ipc.recv() { // Option<IpcMsg>
                    Some(msg) => {
                        println!(">[{}] {:?}", ipc.id, msg);
                    },
                    None => break
                }
            }
            //// send
            //let msg :IpcMsg = IpcMsg{id: ipc.id, msg:"PING".to_string()};
            //let len :i32    = ipc.send(&msg);
            //println!("[{}]> {:?} {}", ipc.id, msg, len);
            //if len < 1 { println!("ipc.send returned {} breaking!", len); break; }
            ::utils::sleep(200);
        }
    } // if isServer
} 


////////////////////////////////////////////////////////////////////////////////
// A fun console 2d gravity/orbital-mechanics simulator
//
fn entity_create_bullet () -> crate::lag::Entity {
    return crate::lag::Entity {
        power: false,
        cast: lag::EntityCast::BULLET,
        age: 0,
        shape : lag::Shape {
            vertices_original : vec!([  0.0,   0.0,  0.0,  1.0]),
            vertices: vec!(),
            color: vec!(15),
            style: vec!(0),
        },
        location: [0.0, 0.0, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0, 0.0],
        angle: 0.0,
        vangle: 0.0
    };
}

fn entity_create_star () -> lag::Entity {
    return lag::Entity {
        power: false,
        cast: lag::EntityCast::STAR,
        age: 0,
        shape : lag::Shape {
            vertices_original : vec!([  0.0,   0.0,  0.0,  1.0]),
            vertices: vec!(),
            color: vec!(15),
            style: vec!(0),
        },
        location: [0.0, 0.0, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0, 0.0],
        angle: 0.0,
        vangle: 0.0
    };
}

fn entity_exhaust_revive (ents :&mut lag::Entities, dir: char) {
    let iship = 2;
    let mut ibullet = 0;
    for i in 3..=7 {
        if !ents[i].power {
            ibullet = i;
            break;
        }
    }
    if ibullet == 0 { return }

    ents[ibullet].power = true;
    ents[ibullet].age = 0;

    ents[ibullet].location =
       [ ents[iship].location[0], ents[iship].location[1], ents[iship].location[2], ents[iship].location[3]];

    match dir {
      'A'=>ents[ibullet].velocity = [  0.0, -1.0, 0.0, 0.0 ],
      'B'=>ents[ibullet].velocity = [  0.0,  1.0, 0.0, 0.0 ],
      'C'=>ents[ibullet].velocity = [  1.0,  0.0, 0.0, 0.0 ],
      'D'=>ents[ibullet].velocity = [ -1.0,  0.0, 0.0, 0.0 ],
      _=>()
    }
}

pub fn mainGravity () {
    let mut power = true;
    let mut tick :usize = 1;
    let mut tb = term::Tbuff::new();
    let mut entities = lag::Entities::new();

    let mut sun = entity_create_star();
    sun.power = true;
    sun.location[1] = 0.0;
    sun.velocity[0] = 0.0;
    sun.shape.color[0] = 11;
    entities.push(sun);

    let mut bullet = entity_create_star();
    bullet.power = true;
    bullet.location[1] = 48.0;
    bullet.velocity[0] = 1.05;
    bullet.shape.color[0] = 10;
    entities.push(bullet);

    let mut bullet2 = entity_create_star();
    bullet2.power = true;
    bullet2.location[1] = 48.0;
    bullet2.velocity[0] = 1.05;
    bullet2.velocity[1] = 0.0;
    bullet2.shape.color[0] = 9;
    entities.push(bullet2);

    // Entity 2-6 - bullets
    for i in 3..=7 {
        let mut bullet = entity_create_bullet();
        bullet.power = false;
        entities.push(bullet);
    }

    // Non-terminal Visualization
    let mut width :u32 = 800;
    let mut height :u32 = 600;
    let mut window: PistonWindow =
        piston_window::WindowSettings::new( "ASCIIRhOIDS", [width, height])
            .exit_on_esc(true)
            .decorated(true)
            .transparent(true)
            .build()
            .unwrap();

    while power {
        for ch in tb.getc().chars() {
            match ch {
              // Quit
              'q' => power = false,
              'A' => {entities[2].velocity[1] +=  0.1; entity_exhaust_revive(&mut entities, 'A'); },
              'B' => {entities[2].velocity[1] += -0.1; entity_exhaust_revive(&mut entities, 'B'); },
              'C' => {entities[2].velocity[0] += -0.1; entity_exhaust_revive(&mut entities, 'C'); },
              'D' => {entities[2].velocity[0] +=  0.1; entity_exhaust_revive(&mut entities, 'D'); },
              _ => ()
            }
        }

        let mut xdist :f32 = entities[1].location[0];
        let mut ydist :f32 = entities[1].location[1];
        let mut distsquared :f32 = xdist*xdist + ydist*ydist;
        let mut dist :f32 = distsquared.sqrt();
        let mut f :f32 = -55.0 / distsquared;
        let mut fx :f32 = f * xdist/dist;
        let mut fy :f32 = f * ydist/dist;
        entities[1].velocity[0] += fx;
        entities[1].velocity[1] += fy;

        xdist = entities[2].location[0];
        ydist = entities[2].location[1];
        distsquared = xdist*xdist + ydist*ydist;
        dist = distsquared.sqrt();
        f = -55.0 / distsquared;
        fx = f * xdist/dist;
        fy = f * ydist/dist;
        entities[2].velocity[0] += fx;
        entities[2].velocity[1] += fy;

        entities.tick_and_transform().expire_bullet();
        tick += 1;
        tb.reset(tick);
        //tb.draw_background_sinies(tick as i32);
        entities.scale_to_terminal_origin_center(&mut tb).draw_shapes(&mut tb);

        while let Some(event) = window.next() {
            if event.render_args() != None {
                width = event.render_args().unwrap().window_size[0] as u32;
                height = event.render_args().unwrap().window_size[1] as u32;
                window.draw_2d(
                    &event,
                    | context, graphics, _device | {
                        clear([0.0, 0.0, 0.0, 1.0], graphics);
                        //entities.plot_shapes(context, graphics);
                        tb.dumpPiston(context, graphics); // Dump the terminal buffer's buffer to stdout
                    }
                );
                break;
            }
        }
        tb.dump(); // Render the finalized terminal buffer.

        print!("\x1b[H\x1b[0;1m{:?} {:?}", entities[1].location, entities[1].velocity);
        print!("\x1b[H\x1b[0;1m{:?}", tick);
        ::utils::flush();
        ::utils::sleep(10);
    }
    tb.done(); // Reset system terminal the way we found it
}

fn main_maze () {
    ::std::println!("== {}:{} ::{}::main_maze() ====", std::file!(), core::line!(), core::module_path!());
    let mut plotter = plotter::Plotter::new();
    let bplotter =
        Box::new( move |hm: &utils::HashMapDeterministic| { plotter.renderhash(hm); } );
    ::maze::start(6, 100, bplotter);
}

pub fn main () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    //crate::mainAsteroid(); // ??? Is there a symbol to explicitly reference the root module or is "crate" and other modules the only symbols?  A: There are only crates and they canonically start with :: and create is the crate representing the current crate.
    //crate::mainGravity();
    //crate::main_maze();
    crate::learn::main();
    //crate::matrix::main();
    //utils::obj::test();
}