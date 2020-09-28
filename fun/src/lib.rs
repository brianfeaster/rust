#![allow(dead_code, unused_imports, unused_variables, non_snake_case)]
mod term;
mod ipc;
mod lag;
mod asciirhoids;

//use crate::ipc::{Ipc};
//use crate::term::{Tbuff}; // Module path prefix "crate::"" and "self::"" are pedantic.
//use crate::lag::{Shape, Entity, Entities, EntityCast};

use ::std::{
    fs,
    //collections::{HashMap},
    net::TcpListener,
    sync::mpsc::{channel, Receiver}};
use ::serde::{Serialize, Deserialize};
use ::serde_json::{self as sj, Value, from_str, to_string_pretty};


/// Enhance Tbuff struct with more useful methods.
/// 
impl crate::term::Tbuff {

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


////////////////////////////////////////////////////////////////////////////////
// A fun console Asteroids game
//
pub fn mainAsteroid () {
    let (txipc, rxipc) = channel::<crate::ipc::Ipc>(); // 
    let doit :bool = true;
    match TcpListener::bind("127.0.0.1:7145") {
        Ok(l) => crate::ipc::server(l, txipc),
        Err(_) => crate::ipc::client(txipc),
    }
    if doit { 
        asciirhoids::asciiteroids(rxipc); // Channel of Ipc objects
    } else {
        let mut ipc :crate::ipc::Ipc = rxipc.recv().unwrap();
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
            ::util::sleep(200);
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
        ::util::flush();
        ::util::sleep(10);
    }
    tb.done(); // Reset system terminal the way we found it
}


////////////////////////////////////////////////////////////////////////////////
// Play with json
//

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

//mod fun;
//pub fn callFun () { fun::main(); }