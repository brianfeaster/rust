// External libraries -- Specified by Cargo
use ::std::{
    //fs,
    //collections::{HashMap},
    //net::TcpListener,
    sync::mpsc::{Receiver}};
use ::rand::Rng;
use ::piston_window::*;

// Local workspace member libraries -- Specified in root Cargo.toml
use ::util::{self};
use ::ipc::{Ipc, IpcMsg};

// Local workspace's member's crate's mod-ules -- Specified in main.rs's lib.rs mod
use ::term::{self};
use crate::lag::{self, Shape, EntityCast, Entity, Entities};

fn load_polygon (filename :&str) -> (Vec<[f32; 4]>, Vec<i32>, Vec<i32>) {

    let mut points = 
        crate::learn::fun_read_poly_file(filename).into_iter().map( |p|
            [p.2, p.1 ,0.0, 1.0]
        ).collect::<Vec<[f32; 4]>>();
    let mut colors :Vec<i32> = vec!();
    let mut styles :Vec<i32> = vec!();

    points.push(points[0]); // Complete the cycle with another bogus first point.
    for p in &points {
        colors.push(15);
        styles.push(1);
    }
    colors[points.len() - 1] = 0;
    styles[points.len() - 1] = 0;

    return (points, colors, styles);
}

fn entity_create_ship () -> Entity {
    let (mut points, colors, styles) = load_polygon("data/ship.dat");
    lag::scale(&mut points, &[0.1, 0.15]);
    return Entity {
        power: true,
        cast: EntityCast::PLAYER,
        age: 0,
        shape : Shape {
            vertices_original :points,
            /*vertices_original : vec!(
              [  0.0,  -0.1,  0.0,  1.0],
              [  0.05,   0.1,  0.0,  1.0],
              [  0.0,   0.0,  0.0,  1.0],
              [ -0.05,   0.1,  0.0,  1.0],
              [  0.0,  -0.1,  0.0,  1.0]),*/
            vertices: vec!(),
            color: colors, //vec!(15, 15, 15, 15, 0),
            style: styles, //vec!(1,  1,  1,  1, 0),
        },
        location: [0.0, 0.0, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0, 0.0],
        angle: 0.0,
        vangle: 0.0
    };
}

fn entity_create_bullet () -> Entity {
    return Entity {
        power: false,
        cast: EntityCast::BULLET,
        age: 0,
        shape : Shape {
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


fn entity_bullet_revive (ents :&mut Entities) {
    let iship = 0;  // Ship index location set on revived bullet
    let mut ibullet = 0;
    for i in 1..=5 {
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

    ents[ibullet].velocity =
       [ ents[iship].velocity[0] + 0.05 * ents[iship].angle.sin(),
         ents[iship].velocity[1] - 0.05 * ents[iship].angle.cos(),
         0.0,
         0.0 ];
}


// Asteroid polygon vertice should go counter-clockwise as
// that's how the polygon is split into traianges for collision's
// right hand rule normal calculations.
fn entity_create_asteroid (xp :f32, yp :f32, vx :f32, vy :f32) -> Entity {
    let (points, colors, styles) = load_polygon("data/erik.dat");
    let e = Entity {
        power: true,
        cast: EntityCast::ENEMY,
        age: 0,
        shape : Shape {
            vertices_original : points /*vec!(
              [  0.0,   0.0,  0.0,  1.0],
              [  0.6,   0.0,  0.0,  1.0],
              [  0.7,  -0.3,  0.0,  1.0],
              [  0.5,  -0.5,  0.0,  1.0],
              [  0.0,  -0.4,  0.0,  1.0],
              [ -0.5,  -0.5,  0.0,  1.0],
              [ -0.8,  -0.2,  0.0,  1.0],
              [ -0.6,   0.2,  0.0,  1.0],
              [ -0.8,   0.4,  0.0,  1.0],
              [ -0.5,   0.6,  0.0,  1.0],
              [ -0.4,   0.8,  0.0,  1.0],
              [  0.5,   0.6,  0.0,  1.0],
              [  0.7,   0.3,  0.0,  1.0],
              [  0.6,   0.0,  0.0,  1.0],
            )*/,
            vertices: vec!(),
            color: colors, // vec!(0, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 0),
            style: styles, //vec!(0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0),
        },
        location: [xp, yp, 0.0, 0.0],
        velocity: [vx/2.0, vy/2.0, 0.0, 0.0],
        angle: 0.0,
        vangle: 0.01
    };
    //scale(&mut e.shape.vertices_original, &[2.0, 3.0]);
    return e;
} // entity_create_asteroid

/*
fn entity_create_line () -> Entity {
    return Entity {
        power: true,
        age: 0,
        shape : Shape {
            vertices_original : vec!(
              [ -10.0,  10.0,  0.0,  1.0],
              [  10.0,   0.0,  0.0,  1.0]),
            vertices: vec!(),
            color: vec!(7, 1),
            style: vec!(1, 0),
        },
        location: [0.0, 0.0, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0, 0.0],
        angle: 0.0,
        vangle: 0.01
    };
}
*/

/// Message describing collision between bullet and a triangle.
struct Collision {
    asteroid: usize,
    edge: usize,
    bullet: usize
}

fn get_collisions (entities : &Entities) -> Vec<Collision>{
    let mut cmds : Vec<Collision> = vec![];
    //for mut asteroid in entities.iter_type(EntityCast::BULLET) {
    for ia in 7..entities.entities.len() { // Asteroids
        if !entities[ia].power { continue } // Ignore disabled asteroids
        for ib in 2..=6 { // Bullets
            if !entities[ib].power { continue } // Ignore disabled bullets
            for iae in 1..entities[ia].shape.vertices.len() { // Vertices of the first asteroid
                if entities[ia].shape.color[iae] == 0 { continue }
                if lag::linein(&entities[ia].shape.vertices[0],  // Center
                          &entities[ia].shape.vertices[iae], // Asteroid line
                          &entities[ia].shape.vertices[iae+1], // Asteroid line
                          &entities[ib].location) {         // bullet location
                  cmds.push(Collision{asteroid:ia, edge:iae, bullet:ib});
                } // if
            } // for
        } // for
    };
    return cmds;
}

#[derive(Debug)]
struct Stats {
   countmsg: i32,
   countmsgbad: i32,
}

pub fn asciiteroids (rx: Receiver<Ipc>) {
    let mut power = true;
    let mut pause = false;
    let mut tick :usize = 1;
    let mut tb = ::term::Tbuff::new();
    let mut entities  = Entities::new();
    let mut rng = rand::thread_rng();
    let mut ipc :Option<Ipc> = None;
    let mut buttonLeft :bool = false;
    let mut buttonRight :bool = false;
    let mut buttonUp :bool = false;
    let mut buttonFire :bool = false;
    let mut buttonAction :bool = false;

    // Entity 0-1 - ships
    entities.push(entity_create_ship());
    entities.push(entity_create_ship());

    // Scale the first two entites, which should be the two player ships.
    entities.iter_mut().take(2).map(|e|lag::scale(&mut e.shape.vertices_original,&[0.1,0.1])).count();

    // Entity 2-6 - bullets
    for i in 2..=6 {
        let mut bullet = entity_create_bullet();
        bullet.power = false;
        entities.push(bullet);
    }

    // Entity 7 - asteroid
    let vx0 = rng.gen_range(-0.3..0.3);
    let vy0 = rng.gen_range(-0.3..0.3);
    let vx1 = rng.gen_range(-0.3..0.3);
    let vy1 = rng.gen_range(-0.3..0.3);
    //entities.push(entity_create_asteroid(-50.0, vx0, vy0));
    entities.push(entity_create_asteroid(0.0, 0.0, 0.0, 0.0));
    //entities.push(entity_create_asteroid(50.0, vx1, vy1));

    // Scale the asteroid just created.
    entities.iter_mut().skip(7).take(1).map(|e|lag::scale(&mut e.shape.vertices_original,&[0.1, 0.1])).count();

    let mut stats = Stats{countmsg:0, countmsgbad:0};

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

    while tick < 10000 && power {
        if ipc.is_none() { ipc = rx.try_recv().ok() }

        let count_bad_sends = ipc.iter_mut().map( |ipc| {
            ipc.send(
                &IpcMsg{id:ipc.id,
                    msg:format!("{} {} {}", entities[0].location[0], entities[0].location[1], entities[0].angle)})
        }).filter(|v| *v == -1).count();

        if 0 < count_bad_sends { ipc = None; }

        // Handle other player messages
        ipc.iter_mut().map( |ipc| {
            let mut counter = 0;
            //print!("\x1b[0H\x1b[0;1m\n");
            loop {
                counter += 1;
                let msg = ipc.recv();
                //println!("\x1b[0;31m{:?} \x1b[32m{} \x1b[33m{:?}\x1b[0m {:?}", ipc.buff, ipc.stagei, ipc.stage, msg);
                match msg {
                    Some(msg) => {
                        stats.countmsg += 1;
                        println!("\x1b[0;1m>[{}] {:?} {}", ipc.id, msg, counter);
                        if msg.id < 0 { println!("ipc.recv returned msg.id {} breaking!", msg.id); break; }
                        let p2 = msg.msg.trim()
                                .split(" ") 
                                .map(|s| s.parse::<f32>().unwrap_or(0.0))
                                .collect::<Vec<_>>();
                        if p2.len() == 3 {
                            entities[1].location[0] = p2[0];
                            entities[1].location[1] = p2[1];
                            entities[1].angle = p2[2];
                        }
                    },
                    None => {
                        println!("\x1b[0;1m>[{}] NONE {}", ipc.id, counter);
                        stats.countmsgbad += 1;
                        break
                    }
                }
            }
        }).count(); // ipc recv loop

        for ch in tb.getc().chars() {
            match ch {
              // Quit
              'q' => power = false,
              'p' => pause = !pause,
              // Rotate
              'j' | 'D' => entities[0].angle -= 0.1,
              'l' | 'C' => entities[0].angle += 0.1,
              // Shoot
              ' ' | 'J' | 'L' | 'I' | 'B' | 'x' => {
                  entity_bullet_revive(&mut entities);
              },
              'a' => {
                    entities.push(entity_create_asteroid(0.0, 0.0, rng.gen_range(-0.01..0.01), rng.gen_range(-0.01..0.01)));
                    let cnt = entities.entities.len() - 1;
                    lag::scale(&mut entities.entities[cnt].shape.vertices_original, &[0.1, 0.1]);
             },
              // Accelerate
              'i' | 'A' | 'z' => {
                   entities[0].velocity[0] += 0.0001 * entities[0].angle.sin();
                   entities[0].velocity[1] -= 0.0001 * entities[0].angle.cos();
              },
              _ => ()
            } // match
        } // for
        //if !power { break; } // Player hit quit.
  
        //if 0 == tick % 10  { entity_bullet_revive(&mut entities) } // Auto bullet firing
  
        entities
            .tick_and_transform()
            .expire_bullet();
  
        // Acquire all colissions betwen bullets and asteroids.
        for Collision{asteroid:a, edge:e, bullet:b} in get_collisions(&entities) {
            entities[b].power = false; // Disable bullet in any colission
            let mut ass = &mut entities[a];
            if 1 < ass.age  {
                ass.power = false;  // Smallest asteroid goes away
            } else {
                // Asteroid gets smaller and duplicated
                ass.age += 1;
                let age = ass.age;
                lag::scale(&mut ass.shape.vertices_original, &[ 0.5, 0.5, 1.0]);
    
                let mut ass2 =
                    entity_create_asteroid(
                        ass.location[0], 
                        ass.location[1], 
                        rng.gen_range(-0.01..0.01),
                        rng.gen_range(-0.01..0.01));
                lag::scale(&mut ass2.shape.vertices_original, &[0.1, 0.1]);
                ass2.age = age;
                let newscale = 0.5_f32.powi(age);
                lag::scale(&mut ass2.shape.vertices_original, &[ newscale, newscale, 1.0]);
                entities.push(ass2);
            }
        }
  
        tick += 1;
        tb.reset(tick);
        //tb.draw_background_sinies(tick as i32);
        //tb.draw_axis();
        entities
            //.scale_to_terminal(&mut tb)
            .wrap(&mut tb)
            .scale_to_viewport(&mut tb, width, height)
            //.draw_shapes(&mut tb) // Draw shapes to framebuffer
        ;

        while let Some(event) = window.next() {
            if event.button_args() != None {
                //println!("asciiteroids() {:?}", event);
                if event.button_args().unwrap().state == ::piston_window::ButtonState::Press {
                    match event.button_args().unwrap().scancode {
                       Some(0) => buttonAction = true,
                       Some(12) => power = false,
                       Some(38) => buttonLeft = true,
                       Some(37) => buttonRight = true,
                       Some(34) => buttonUp = true,
                       Some(6) => buttonUp = true,
                       Some(49) => buttonFire = true,
                       Some(7) => buttonFire = true,
                       Some(35) => pause = !pause,
                       _ => ()
                    }
                }
                if event.button_args().unwrap().state == ::piston_window::ButtonState::Release {
                    match event.button_args().unwrap().scancode {
                       Some(0) => buttonAction = false,
                       Some(12) => power = false,
                       Some(38) => buttonLeft = false,
                       Some(37) => buttonRight = false,
                       Some(34) => buttonUp = false,
                       Some(6) => buttonUp = false,
                       Some(49) => buttonFire = false,
                       Some(7) => buttonFire = false,
                       _ => ()
                    }
                }
            }
            //if event.text_args() != None { tb.pushc(event.text_args().unwrap()); }

            if event.render_args() != None {
                width = event.render_args().unwrap().window_size[0] as u32;
                height = event.render_args().unwrap().window_size[1] as u32;
                if pause { break; }
                window.draw_2d(
                    &event,
                    | context, graphics, _device | {
                        clear([0.0, 0.0, 0.0, 1.0], graphics);
                        entities.plot_shapes(context, graphics);
                        //polygon([0.0, 1.0, 1.0, 1.0], &[[0.1, 0.1], [-0.1, -0.1], [-0.1, 0.1]], math::rotate_radians(1.0), graphics);
                        //tb.dump(context, graphics); // Dump the terminal buffer's buffer to stdout
                    }
                );
                break;
            }
        }
        
        // Stuff terminal object with key presses based on key press events from graphic window.
        if tick % 1 == 0 {
            if buttonAction == true {
                tb.pushc("a".to_string());
                buttonAction = false;
            }
            if buttonLeft == true { tb.pushc("j".to_string()); }
            if buttonRight == true { tb.pushc("l".to_string()); }
            if buttonUp == true { tb.pushc("z".to_string()); }
            if buttonFire == true { tb.pushc("x".to_string()); }
        }

  
        //// Details of entities
        //let r = (entities.iter_type(EntityCast::PLAYER).count(),
        //         entities.iter_type(EntityCast::BULLET).count(),
        //         entities.iter_type(EntityCast::ENEMY).count());
        //print!("\x1b[H\x1b[0;36m {:?}", r);
  
        // Details ^[[2F reverse CRLF
        //print!("\x1b[2J\x1b[H\x1b[0;1m");
        //entities.iter().map( |e|
        //   println!("{:6.2} {:6.2} {:?}", e.location[0], e.location[1], e.cast)
        //).count();
  
        //util::flush();
        //util::sleep(0);
    } // while tick && power
    tb.done(); // Reset system terminal the way we found it
}
