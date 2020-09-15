#![allow(dead_code, unused_imports, unused_variables, non_snake_case)]

mod util;
mod term;
use crate::term::{Tbuff}; // Module path prefix "crate::"" and "self::"" are pedantic.
use rand::Rng;

use std::fmt;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::str;
use std::fs;
use std::collections::{HashMap};
use serde::{Serialize, Deserialize};
use serde_json::{self as sj, Value, from_str, to_string_pretty};

/// Enhance Tbuff struct with more useful methods.
/// 
impl term::Tbuff {

    fn clear (self  :&mut term::Tbuff,
            bg    :i32,
            fg    :i32,
            ch    :char) -> &mut Self {
        for y in 0..(self.rows()-0) {
            for x in 0..self.cols() {
                self.set(x, y, bg, fg, ch);
            }
        }
        return self;
    }

    fn draw_axis (self :&mut term::Tbuff) -> &mut Self{
        for y in 0..(self.rows()-0) {
            self.set(0, y, 0, 8, '|'); 
        }
    
        for x in 0..self.cols() {
            self.set(x, 0, 0, 8, '-');
        }
        self.set(0, 0 , 0, 8, '+');
        return self;
    }

    fn draw_background_sinies (
            self : &mut term::Tbuff,
            z    : i32) -> &mut Self {
        for y in 0..self.rows() {
            let h = 0.1 + 0.3 * ( 0.1 * (z as f32)).sin();
            let g = (6.28 / (24.0 + h) * (y as f32 * 95.0 + z as f32)).sin();
            for x in 0..self.cols() {
                let k = 0.3 + 0.3 * ( 0.1 * (z as f32)).sin();
                let j = (6.28 / (48.0 + k) * (y as f32 * 95.0 + x as f32+ z as f32)).sin();
                //let n = ((g + j) / 2.0 * 127.99 + 128.0) as i32;
                //let bg = (n/3) % 256;
                //self.set(x, y, bg*65536 + bg*256 + bg, bg*65536 + bg*256 + bg, ' '); 
                let n = ((g + j) / 2.0 * 11.99 + 12.0) as i32;
                let bg = (n/3) % 24 + 232;
                self.set(x, y, bg, bg, '.'); 
            }
        }
        return self;
    }

} // impl term::Tbuff



/// 3d Stuff
/// 
type Vertex = [f32; 4];  // Homogeneous 

type Vertices = Vec<Vertex>;

fn rotate (vs : &mut Vertices, ang : f32) {
    for v in vs {
      let y = v[1];
      let x = v[0];
      v[0] =   x * ang.cos() - y * ang.sin();
      v[1] =   x * ang.sin() + y * ang.cos();
    }
}

fn translate ( vs : & mut Vertices,
              loc : & Vertex) {
    for v in vs {
      v[0] += loc[0];
      v[1] += loc[1];
    }
}

fn scale (vs : &mut Vertices, s :&[f32]) {
    for v in vs {
      v[0] *= s[0];
      v[1] *= s[1];
    }
}

// Is point 'pt' in front of behind line 'v0->v1'
fn linein (v0 :& Vertex,
           v1 :& Vertex,
           v2 :& Vertex,
           pt :& Vertex) -> bool{
    return
        0.0 < (v1[1]-v0[1])*(pt[0] - v0[0]) - (v1[0]-v0[0])*(pt[1] - v0[1]) &&
        0.0 < (v2[1]-v1[1])*(pt[0] - v1[0]) - (v2[0]-v1[0])*(pt[1] - v1[1]) &&
        0.0 < (v0[1]-v2[1])*(pt[0] - v2[0]) - (v0[0]-v2[0])*(pt[1] - v2[1])
}

/// Shape - collection of lines and points to render
struct Shape {
    vertices_original : Vertices,
    vertices: Vertices,
    color : Vec<i32>,
    style : Vec<i32>, // 0 point, 1 line to next vertex
}

impl Shape {
    fn reset (&mut self) {
        self.vertices = self.vertices_original.clone();
    }
}

/// Shape with motion
#[derive(PartialEq, Debug)]
enum EntityCast {
    PLAYER, BULLET, ENEMY, STAR
}
struct Entity {
    power: bool,  // is this alive/dead
    cast: EntityCast,
    age: i32,     // age in ticks
    shape: Shape, // The meat popsicle
    location: Vertex, // Cartesian location
    velocity: Vertex, // Where's its headed
    angle: f32,  // Current rotation
    vangle: f32, // Where it's rotating
}

impl Entity {

    fn tick_and_transform (&mut self) -> &mut Self {
        if !self.power { return self; }
        //self.age += 1;
        self.shape.reset();
        self.location[0] += self.velocity[0];
        self.location[1] += self.velocity[1];
        self.angle += self.vangle;
        scale(&mut self.shape.vertices, &[2.0, 2.0]);
        rotate(&mut self.shape.vertices, self.angle);
        translate(&mut self.shape.vertices, &self.location);
        return self;
    }

    fn scale_to_terminal (&mut self, tb :&Tbuff) -> &mut Self{
        if !self.power { return self; }
        // Scale to screen size
        let twidth = tb.cols() as f32;
        let theight = tb.rows() as f32;
        scale(&mut self.shape.vertices, &[twidth/170.0, theight/100.0]);
        return self;
    }

    fn scale_to_terminal_origin_center (self: &mut Entity, tb :&Tbuff, obj_center: &Vertex) -> &mut Self {
        if !self.power { return self; }
        // Scale to screen size
        let twidth = tb.cols() as f32;
        let theight = tb.rows() as f32;
        scale(&mut self.shape.vertices, &[twidth/(170.0*2.0), theight/(100.0*2.0)]);
        translate(&mut self.shape.vertices,
                  &[(tb.cols() as f32 / 2.0) - obj_center[0] * twidth/(170.0*2.0),
                    (tb.rows() as f32 / 2.0) - obj_center[1] * theight/(100.0*2.0),
                    0.0,
                    0.0]);
        return self;
    }

    fn draw_spokes (
            self :&mut Entity,
            tb   :&mut term::Tbuff)  -> &mut Self {
        let shape = &self.shape;
        for v in 0..shape.vertices.len() {
            if v == 0 || (shape.color[v-1] == shape.color[v] ) { continue }
            let vs1 = shape.vertices[0];
            let vs2 = shape.vertices[v];
            let mut x = vs1[0] as i32;
            let mut y = vs1[1] as i32;
            for [xinc, yinc, ch] in util::Walk::new(&vs1[0..2], &vs2[0..2]) {
                x += xinc;
                y += yinc;
                tb.set(x, y, 0, 1, ch as u8 as char);
            }
        } // while
        self
    } // draw_spokes

    fn draw_shape (
            self : &mut Entity,
            tb  :&mut term::Tbuff) -> &mut Self {
        if !self.power { return self; }
        let shape = &self.shape;
        for v in 0..shape.vertices.len() {
            if shape.color[v] == 0 { continue }
            if 0 == shape.style[v] {
                tb.set(
                     shape.vertices[v][0] as i32,
                     shape.vertices[v][1] as i32,
                     0, shape.color[v], '*');
            }  else if 1 == shape.style[v] {
               tb.line(&shape.vertices[v], &shape.vertices[v+1], '@', shape.color[v]);
            } // else
        } // while
        return self;
    } // fb_draw_asteroid 

} // impl Entity

/// Entities - Container of all entities
struct Entities {
   entities :Vec<Entity>
}

impl std::ops::IndexMut<usize> for Entities {
    fn index_mut (&mut self, i:usize) -> &mut Entity {
      &mut self.entities[i]
    }
}

impl std::ops::Index<usize> for Entities {
    type Output = Entity;
    fn index (&self, i:usize) -> &Entity {
      &self.entities[i]
    }
}

impl IntoIterator for Entities {
    type Item = Entity;
    type IntoIter = ::std::vec::IntoIter<Entity>;
    fn into_iter(self) -> Self::IntoIter {
        self.entities.into_iter()
    }
}

impl<'s> IntoIterator for &'s Entities {
    type Item = &'s Entity;
    type IntoIter = ::std::slice::Iter<'s, Entity>;
    fn into_iter(self) -> Self::IntoIter {
        self.entities.iter()
    }
}
impl<'s> IntoIterator for &'s mut Entities {
    type Item = &'s mut Entity;
    type IntoIter = ::std::slice::IterMut<'s, Entity>;
    fn into_iter(self) -> Self::IntoIter {
        self.entities.iter_mut()
    }
}

impl Entities {

    pub fn new() -> Self {
        Entities{entities:vec!()}
    }
    pub fn push(&mut self, e:Entity) -> &Self {
        self.entities.push(e);
        self
    }

    pub fn iter_type <'a> (&'a self, cast :EntityCast) -> Box<dyn Iterator<Item = &'a Entity> + 'a> {
        Box::new(self.entities.iter().filter( move |& e| e.cast == cast ))
    }

    pub fn iter(&self) -> std::slice::Iter<Entity> {
        self.entities.iter()
    }
    pub fn iter_mut(self :&mut Entities) -> std::slice::IterMut<Entity> {
        self.entities.iter_mut()
    }

    fn tick_and_transform (&mut self) -> &mut Self {
          self.iter_mut().map(Entity::tick_and_transform).count();
          self
    }

    fn scale_to_terminal (&mut self, tb :&Tbuff) -> &mut Self {
          self.iter_mut().map( |e|e.scale_to_terminal(tb) ).count();
          self
    }
    fn scale_to_terminal_origin_center (&mut self, tb :&Tbuff) -> &mut Self {
          let center_loc = self.entities[1].location.clone();
          self.iter_mut().enumerate().map( |(i,e)|e.scale_to_terminal_origin_center(tb,
            //if 1 == i { &[0.0, 0.0, 0.0, 0.0] } else { &center_loc }
            &[0.0, 0.0, 0.0, 0.0]
            //&center_loc
          )).count();
          self
    }


    // Hide bullet after a while (limited life span)
    fn age_bullet (&mut self) -> &mut Self {
        for mut e in self.iter_mut() {
            if e.cast== EntityCast::BULLET {
                e.age += 1;
                if 25 < e.age {
                    e.power = false
                }
            }
        }
        self
    }

    fn draw_shapes (&mut self, tb :& mut term::Tbuff) {
        for e in self.iter_mut() {
            //e.draw_spokes(tb);
            e.draw_shape(tb);
        }
    }
} // impl Entities

////////////////////////////////////////////////////////////////////////////////

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

fn entity_create_star () -> Entity {
    return Entity {
        power: false,
        cast: EntityCast::STAR,
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

fn entity_exhaust_revive (ents :&mut Entities, dir: char) {
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
                if linein(&entities[ia].shape.vertices[0],  // Center
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

fn asciiteroids (rx: Receiver<Ipc>) {
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
                scale(&mut ass.shape.vertices_original, &[ 0.5, 0.5, 1.0]);
    
                let mut ass2 =
                    entity_create_asteroid(
                        ass.location[0], 
                        ass.location[1], 
                        rng.gen_range(-0.6, 0.6),
                        rng.gen_range(-0.6, 0.6));
                ass2.age = age;
                let newscale = 0.5_f32.powi(age);
                scale(&mut ass2.shape.vertices_original, &[ newscale, newscale, 1.0]);
                entities.push(ass2);
            }
        }
  
        tick += 1;
        tb.reset(tick);
        //tb.draw_background_sinies(tick as i32);
        //tb.draw_axis();
        entities.scale_to_terminal(&mut tb).draw_shapes(&mut tb);
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

// Bidirectional pipe of bytes.
// When recv is called, bytes are written to buff, not aways at position 0
struct Ipc {
    stream: TcpStream,
    id: i32,
    buff: String,  // Where incoming bytes are written to always at [0..]
    stage: String, // Buffer spillover
    stagei: usize, // Staging buffer read index
}

// 
struct IpcMsg {
    id: i32, // Id of agent that sent  (or my Id when sending)
    msg: String // newline delimited string acquired from last recv
}

impl Ipc {

    pub fn new (stream:TcpStream, id:i32) -> Self {
        crate::Ipc{
            stream : stream,
            id     : id,
            buff   : " ".repeat(64),
            stage  : String::new(),
            stagei : 0}
    } // Ipc::new

    pub fn send (self :&mut crate::Ipc,
                 msg  :&IpcMsg) -> i32 {
        let buff = format!("{} {}\n", msg.id, msg.msg);
        match self.stream.write(buff.as_bytes()) {
            Ok(len) => len as i32,
            Err(e) => { eprintln!("{:?}", e); -1 }
        }
    } // Ipc::sendread.

    // This should snarf bytes into the local buffer until a newline is read.
    // The bytes will be copied to a new string and returned in an IpcMsg.
    //          buff       idx,stage
    //          [____]     0 []     Initial
    //          [a.__]     0 []     Emit "a"    (Read "a.", update tail index, )
    //          [b___]     0 [b]     Partial,return
    //          [cdef]     0 [bcdef] Partial continue
    //          [.f__]     0 [f]    Emit "bcdef"
    //          [g.__]     0 [f]    Emit "fg"
    //          [____]     0 []     Final
    // If last read char is newline, always reset to Initial
    pub fn recv_line (self :&mut Ipc) -> Option<String> {

        // If there's a full msg in staging, update the read index and return the msg.
        // Otherwise staging is empty or has partial message, so can be reset
        // after the next message is acquired.
        let stages :&str = &self.stage[self.stagei..];
        match stages.find("\n") {
           Some(idx) => {
               self.stagei += idx + 1;
               return Some(String::from(&stages[..=idx]));
           }
           None => 0
        };

        // Read bytes from OS into buffer
        let count_buff :usize =
            match self.stream.read( unsafe { self.buff.as_bytes_mut() } ) {
                Ok(count_buff) => {
                    //println!("\x1b[35mREAD {:#?}", count_buff);
                    count_buff
                },
                Err(e) => { 
                    //println!("\x1b[35mREAD {:?}", e);
                    if e.kind() == std::io::ErrorKind::ConnectionReset {
                        self.stream.shutdown(std::net::Shutdown::Both).expect("shutdown failed");
                    }
                    return None;
                 } // e.kind() == std::io::ErrorKind::WouldBlock
            };

        let count_msg :usize = 
            match self.buff[..count_buff].find("\n") {
               Some(idx) => idx + 1,
               None => 0
            };

        let mut new_msg :Option<String> = None;

        //println!("\x1b[36m count_buff {}  count_msg {}", count_buff, count_msg);
        if 1 <= count_msg {
            // Create new message from stage[stagei..] + buff[..count_mg]
            new_msg = Some(
                 String::from(&self.stage[self.stagei..])
                  + &self.buff[..count_msg]);
            // Reset stage
            self.stage.clear();
            self.stagei = 0;
        }
        // Append any extra buffer chars to staging
        self.stage.push_str(&self.buff[count_msg..count_buff]);
        return new_msg;
    } // Ipc::recv_line

    pub fn recv (self :&mut Ipc) -> Option<IpcMsg> {
        //// Clear receive buffer so debugging is easier
        //unsafe{self.buff.as_bytes_mut()}.iter_mut().map( |c| { *c = b' '; } ).count();
        let theline = self.recv_line();
        match theline {
            Some(msg) => {
                let (idstr, msgstr) : (&str, &str) = msg.split_at(msg.find(' ').unwrap_or(0));
                let newid = idstr.parse::<i32>().unwrap_or(-1);
                if 0 <= newid  {
                    Some(IpcMsg{  id: newid,
                                 msg: String::from(msgstr.trim())})
                } else {
                    Some(IpcMsg{  id: -1,
                                 msg: String::from(msg)})
                }
            },
            None => None
        }
    } // Ipc::recv
}

impl fmt::Debug for IpcMsg {
    fn fmt(self:&IpcMsg, f: &mut fmt::Formatter
        ) -> fmt::Result
    {
        f.debug_struct("IpcMsg")
         .field("id", &self.id)
         .field("msg",  &self.msg)
         .finish()
    }
}

// TODO: Encapsulate server and client into a single agent
// that handles connectivity to everyone else.  Star topology.

fn server (listener:TcpListener,  txipc:Sender<Ipc>) {
    thread::spawn( move || {
      // I'm the server.  So wait for a connection, then create the Ipc object for myself.
      // Also notify the client it's player 1 (I'm 0)
      let mut playerCount = 0;
      for stream in listener.incoming() {
          let mut ipc = Ipc::new(stream.unwrap(), playerCount);
          ipc.stream.set_read_timeout(
               std::option::Option::Some(
                   ::std::time::Duration::new(0, 1))).expect("ERROR: set_read_timeout");
          // Broadcast first msg to new client/player: uniqueID and game type
          playerCount += 1;
          let msg :IpcMsg = IpcMsg{id:playerCount, msg:"RUSTEROIDS".to_string()};
          let len :i32    = ipc.send(&msg);
          println!("[{}]> {:?} {}", ipc.id, msg, len);
          txipc.send(ipc).expect("Server unable to send Ipc to local channel.");
        }
      }); // thread lambda
}
// Send the Ipc object through the channel.  The agent will use this to communicate with the remote agent.
fn client (txipc: Sender<Ipc>) {
    let stream = TcpStream::connect("127.0.0.1:7145").unwrap();
    let mut ipc = Ipc::new(stream, -1);
    let msg0 = ipc.recv().unwrap();

    ipc.id = msg0.id;
    println!(">[{}] {:?}", ipc.id, msg0);

    ipc.stream.set_read_timeout(
         std::option::Option::Some(
             ::std::time::Duration::new(0, 1))).expect("ERROR: set_read_timeout");

    txipc.send(ipc).expect("Client unable to send Ipc to local channel.");

}


fn mainAsteroid () {
    let (txipc, rxipc) = channel::<Ipc>(); // 
    let doit :bool = true;
    match TcpListener::bind("127.0.0.1:7145") {
        Ok(l) => server(l, txipc),
        Err(_) => client(txipc),
    }
    if doit { 
        asciiteroids(rxipc); // Channel of Ipc objects
    } else {
        let mut ipc :Ipc = rxipc.recv().unwrap();
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
            util::sleep(200);
        }
    } // if isServer
} 

fn mainGravity () {
    let mut power = true;
    let mut tick :usize = 1;
    let mut tb = term::Tbuff::new();
    let mut entities  = Entities::new();

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

        entities.tick_and_transform().age_bullet();
        tick += 1;
        tb.reset(tick);
        //tb.draw_background_sinies(tick as i32);
        entities.scale_to_terminal_origin_center(&mut tb).draw_shapes(&mut tb);
        tb.dump(); // Render the finalized terminal buffer.
        //print!("\x1b[H\x1b[0;1m{:?} {:?}", entities[1].location, entities[1].velocity);
        //print!("\x1b[H\x1b[0;1m{:?}", tick);
        util::flush();
        util::sleep(10);
    }
    tb.done(); // Reset system terminal the way we found it
}


#[derive(Debug)]
enum MyError {
    IoError(std::io::Error),
    JsonError(json::Error),
    SerdeJsonError(serde_json::Error)
}

impl From<std::io::Error>     for MyError { fn from(error: std::io::Error)    -> Self { MyError::IoError(error) }   }
impl From<json::Error>        for MyError { fn from(error: json::Error)       -> Self { MyError::JsonError(error) } }
impl From<serde_json::Error>  for MyError { fn from(error: serde_json::Error) -> Self { MyError::SerdeJsonError(error) } }

fn mainJson () -> Result<usize, MyError> {
    Ok(
        json::parse(&fs::read_to_string("products.json")?)?
        ["treats"]
        .members()
        .map( |e| println!("{}", e["name"].pretty(1)) )
        .count()
    )
}

// BF

#[derive(Serialize, Deserialize, Debug)]
struct BulkPricing {
    amount : i32,
    totalPrice : f32
}

#[derive(Serialize, Deserialize, Debug)]
struct Treat {
    id: i32,
    name: String,
    imageURL: String,
    price: f32,
    bulkPricing: Option<BulkPricing>
}

#[derive(Serialize, Deserialize, Debug)]
struct Products {
    treats :Vec<Treat>
}

fn mainJsonSerdes () -> Result<usize, MyError> {
    let v :Products //HashMap<String, Vec<sj::Value>>
        = sj::from_str(&fs::read_to_string("products.json")?)?;
    Ok(
        v.treats
        .iter()
        .map( |e| println!("{}", sj::to_string_pretty(&e.name).unwrap()))
        .count()
    )
}

fn main () {
    mainAsteroid();
    //mainGravity();
    //println!("{:?}", mainJson());
    //println!("!!! {:?}", mainJsonSerdes());
    //println!("map {:?}", ('🐘' .. '🐷').map(|x| (|x| x)(x)).collect::<Vec<char>>()); // type std::ops::RangeInclusive

}