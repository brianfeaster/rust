use crate::term::{self};
use ::util::{self};
use crate::lag::{self, Shape, EntityCast, Entity, Entities};
use crate::ipc::{Ipc, IpcMsg};
use ::rand::Rng;

use ::std::{
    //fs,
    //collections::{HashMap},
    //net::TcpListener,
    sync::mpsc::{Receiver}};

fn entity_create_ship () -> Entity {
    return Entity {
        power: true,
        cast: EntityCast::PLAYER,
        age: 0,
        shape : Shape {
            vertices_original : vec!(
              [  3.0,   0.0,  0.0,  1.0],
              [ -1.0,  -2.0,  0.0,  1.0],
              [  3.0,   0.0,  0.0,  1.0],
              [ -1.0,   2.0,  0.0,  1.0],
              [ -1.0,  -2.0,  0.0,  1.0],
              [ -1.0,   2.0,  0.0,  1.0]),
            vertices: vec!(),
            color: vec!(15, 15, 15, 15, 240, 15),
            style: vec!(1,  0,  1,  0, 1, 0),
        },
        location: [50.0, 50.0, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0, 0.0],
        angle: 3.15,
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
    let iship = 0;
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
       [ ents[iship].velocity[0] + 3.0 * ents[iship].angle.cos(),
         ents[iship].velocity[1] + 3.0 * ents[iship].angle.sin(),
         0.0,
         0.0 ];
}


fn entity_create_asteroid (xp :f32, yp :f32, vx :f32, vy :f32) -> Entity {
    let e = Entity {
        power: true,
        cast: EntityCast::ENEMY,
        age: 0,
        shape : Shape {
            vertices_original : vec!(
              [  0.0,   0.0,  0.0,  1.0],
              [  6.0,   0.0,  0.0,  1.0],
              [  7.0,  -3.0,  0.0,  1.0],
              [  5.0,  -5.0,  0.0,  1.0],
              [  0.0,  -4.0,  0.0,  1.0],
              [ -5.0,  -5.0,  0.0,  1.0],
              [ -8.0,  -2.0,  0.0,  1.0],
              [ -6.0,   2.0,  0.0,  1.0],
              [ -8.0,   4.0,  0.0,  1.0],
              [ -5.0,   6.0,  0.0,  1.0],
              [ -4.0,   8.0,  0.0,  1.0],
              [  5.0,   6.0,  0.0,  1.0],
              [  7.0,   3.0,  0.0,  1.0],
              [  6.0,   0.0,  0.0,  1.0],
            ),
            vertices: vec!(),
            color: vec!(0, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 0),
            style: vec!(0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0),
        },
        location: [xp, yp, 0.0, 0.0],
        velocity: [vx/2.0, vy/2.0, 0.0, 0.0],
        angle: 0.0,
        vangle: 0.1
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
            for iae in 1..=12 { // Vertices of the first asteroid
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
    let mut tick :usize = 1;
    let mut tb = term::Tbuff::new();
    let mut entities  = Entities::new();
    let mut rng = rand::thread_rng();
    let mut ipc :Option<Ipc> = None;

    // Entity 0-1 - ships
    entities.push(entity_create_ship());
    entities.push(entity_create_ship());

    // Entity 2-6 - bullets
    for i in 2..=6 {
        let mut bullet = entity_create_bullet();
        bullet.power = false;
        entities.push(bullet);
    }

    // Entity 7 - asteroid
    let vx0 = rng.gen_range(-0.3, 0.3);
    let vy0 = rng.gen_range(-0.3, 0.3);
    let vx1 = rng.gen_range(-0.3, 0.3);
    let vy1 = rng.gen_range(-0.3, 0.3);
    //entities.push(entity_create_asteroid(-50.0, vx0, vy0));
    entities.push(entity_create_asteroid(100.0, 50.0, 0.0, 0.0));
    //entities.push(entity_create_asteroid(50.0, vx1, vy1));

    let mut stats = Stats{countmsg:0, countmsgbad:0};

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
            print!("\x1b[0H\x1b[0;1m\n");
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

        util::flush();
        util::sleep(30);

        for ch in tb.getc().chars() {
            match ch {
              // Quit
              'q' => power = false,
              // Rotate
              'j' | 'D' => entities[0].angle -= 0.1,
              'l' | 'C' => entities[0].angle += 0.1,
              // Shoot
              ' ' | 'J' | 'L' | 'I' | 'z' | 'B' => {
                  entity_bullet_revive(&mut entities);
              },
              'a' => { entities.push(entity_create_asteroid(50.0, 50.0, rng.gen_range(-0.3, 0.3), rng.gen_range(-0.3, 0.3))); },
              // Accelerate
              'i' | 'A' => {
                   entities[0].velocity[0] += 0.1 * entities[0].angle.cos();
                   entities[0].velocity[1] += 0.1 * entities[0].angle.sin();
              },
              _ => ()
            } // match
        } // for
        //if !power { break; } // Player hit quit.
  
        //if 0 == tick % 10  { entity_bullet_revive(&mut entities) } // Auto bullet firing
  
        entities.tick_and_transform().age_bullet();
  
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
                        rng.gen_range(-0.6, 0.6),
                        rng.gen_range(-0.6, 0.6));
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
            .scale_to_terminal(&mut tb)
            .draw_shapes(&mut tb);
        tb.dump(); // Render the finalized terminal buffer.
  
        //// Details of entities
        //let r = (entities.iter_type(EntityCast::PLAYER).count(),
        //         entities.iter_type(EntityCast::BULLET).count(),
        //         entities.iter_type(EntityCast::ENEMY).count());
        //print!("\x1b[H\x1b[0;36m {:?}", r);
  
        // Details ^[[2F reverse CRLF
        print!("\x1b[H\x1b[0;1m{:6.2} {:6.2}\n\n", entities[0].location[0], entities[0].location[1]);
  
    } // while tick && power
    tb.done(); // Reset system terminal the way we found it
}
