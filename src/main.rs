#![allow(dead_code, unused_imports, unused_variables)]

mod util;
mod term;
use crate::term::{Tbuff}; // Module path prefix "crate::"" and "self::"" are pedantic.
use rand::Rng;


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
        for y in 0..(self.rows()-0){
            let h = 0.1 + 0.3 * ( 0.1 * (z as f32)).sin();
            let g = (6.28 / (24.0 + h) * (y as f32 * 95.0 + z as f32)).sin();
            for x in 0..self.cols() {
                let k = 0.3 + 0.3 * ( 0.1 * (z as f32)).sin();
                let j = (6.28 / (48.0 + k) * (y as f32 * 95.0 + x as f32+ z as f32)).sin();
                let n = ((g + j) / 2.0 * 11.99 + 12.0) as i32;
                let bg = (n/3) % 24 + 232;
                self.set(x, y, bg, 0, ' '); 
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
    PLAYER, BULLET, ENEMY
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
        self.age += 1;
        self.shape.reset();
        self.location[0] += self.velocity[0];
        self.location[1] += self.velocity[1];
        self.angle += self.vangle;
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
            for [xinc, yinc] in util::Walk::new(&vs1[0..2], &vs2[0..2]) {
                x += xinc;
                y += yinc;
                tb.set(x, y, 0, 1, '@');
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
                     0, shape.color[v], '@');
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


    // Hide bullet after a while (limited life span)
    fn age_bullet (&mut self) -> &mut Self {
        for mut e in self.iter_mut() {
            if 25 < e.age && e.cast == EntityCast::BULLET {
                e.power = false
            }
        }
        self
    }

    fn draw_shapes (&mut self, tb :& mut term::Tbuff) {
        for e in self.iter_mut() {
            e.draw_spokes(tb);
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
              [ -1.0,   2.0,  0.0,  1.0],
              [  3.0,   0.0,  0.0,  1.0]),
            vertices: vec!(),
            color: vec!(4, 1, 4, 7),
            style: vec!(1, 1, 1, 0),
        },
        location: [100.0, 50.0, 0.0, 0.0],
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
            color: vec!(1),
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
    return Entity {
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
            color: vec!(0, 1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5, 0),
            style: vec!(0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0),
        },
        location: [xp, yp, 0.0, 0.0],
        velocity: [vx/2.0, vy/2.0, 0.0, 0.0],
        angle: 0.0,
        vangle: 0.01
    };
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
    for ia in 6..entities.entities.len() { // Asteroids
        for ib in 1..=5 { // Bullets
            if !entities[ib].power { continue }
            for iae in 1..=12 { // Vertices of the first asteroid
                if entities[ia].shape.color[iae] == 0 { continue }
                if linein(&entities[ia].shape.vertices[0],  // Center
                          &entities[ia].shape.vertices[iae], // Asteroid line
                          &entities[ia].shape.vertices[iae+1],
                          &entities[ib].location) {         // bullet location
                  cmds.push(Collision{asteroid:ia, edge:iae, bullet:ib});
                } // if
            } // for
        } // for
    };
    return cmds;
}

fn asciiteroids () {
    let mut power = true;
    let mut tick :i32 = 0;
    let mut tb = term::Tbuff::new();
    let mut entities  = Entities::new();
    let mut rng = rand::thread_rng();

    // Entity 0 - ship
    entities.push(entity_create_ship());

    // Entity 1-5 - bullets
    for i in 1..=5 {
        let mut bullet = entity_create_bullet();
        bullet.power = false;
        entities.push(bullet);
    }

    // Entity 6 - asteroid
    let vx0 = rng.gen_range(-0.3, 0.3);
    let vy0 = rng.gen_range(-0.3, 0.3);
    let vx1 = rng.gen_range(-0.3, 0.3);
    let vy1 = rng.gen_range(-0.3, 0.3);
    //entities.push(entity_create_asteroid(-50.0, vx0, vy0));
    entities.push(entity_create_asteroid(100.0, 50.0, 0.0, 0.0));
    //entities.push(entity_create_asteroid(50.0, vx1, vy1));

    while tick < 10000 && power {
      tb.reset();
      for ch in tb.getc().chars() {
          match ch {
            // Quit
            'q' => power = false,
            // Rotate
            'j' => entities[0].angle -= 0.1,
            'l' => entities[0].angle += 0.1,
            // Shoot
            ' ' | 'J' | 'L' | 'I' => {
                entity_bullet_revive(&mut entities);
            },
            'a' => { entities.push(entity_create_asteroid(50.0, 50.0, rng.gen_range(-0.3, 0.3), rng.gen_range(-0.3, 0.3))); },
            // Accelerate
            'i' => {
                 entities[0].velocity[0] += 0.1 * entities[0].angle.cos();
                 entities[0].velocity[1] += 0.1 * entities[0].angle.sin();
            },
            _ => ()
          } // match
      } // for
      //if !power { break; } // Player hit quit.

      if 0 == tick % 10  { entity_bullet_revive(&mut entities) } // Auto bullet firing

      tb.clear(0, 0, '.');
      //tb.draw_background_sinies(tick);

      entities.tick_and_transform().age_bullet();

      for Collision{asteroid:a, edge:e, bullet:b} in get_collisions(&entities) {
          entities[a].shape.color[e] = 0;
          entities[b].power = false;
      }

      tick += 1;

      //tb.draw_axis();
      entities.scale_to_terminal(&mut tb).draw_shapes(&mut tb);

      tb.dump(); // Render the finalized terminal buffer.

      // Details of entities
      let r = (entities.iter_type(EntityCast::PLAYER).count(),
               entities.iter_type(EntityCast::BULLET).count(),
               entities.iter_type(EntityCast::ENEMY).count());
      print!("\x1b[2A\n\x1b[0;36m {:?} {:?}", r, rng.gen_range(-40.0, 1.3e4));

      // Details of player1
      //print!("\x1b[2A\n\x1b[0;1;36m {:.2} {:?} {:?}", entities[0].angle, entities[0].location, entities[0].velocity);

      util::flush();
      util::sleep(30);
    } // while tick && power
    tb.done(); // Reset system terminal the way we found it
}

fn main () {
    asciiteroids();
}