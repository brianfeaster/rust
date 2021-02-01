use ::term::{self, Tbuff}; // Module path prefix "crate::"" and "self::"" are pedantic.
use ::utils;
use ::piston_window::*;

/// Linear AlGebra -- 3d Stuff

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

pub fn scale (vs : &mut Vertices, s :&[f32]) {
    for v in vs {
      v[0] *= s[0];
      v[1] *= s[1];
    }
}

// Is point 'pt' in front of behind line 'v0->v1'
pub fn linein (v0 :& Vertex,
           v1 :& Vertex,
           v2 :& Vertex,
           pt :& Vertex) -> bool{
    return
        0.0 < (v1[1]-v0[1])*(pt[0] - v0[0]) - (v1[0]-v0[0])*(pt[1] - v0[1]) &&
        0.0 < (v2[1]-v1[1])*(pt[0] - v1[0]) - (v2[0]-v1[0])*(pt[1] - v1[1]) &&
        0.0 < (v0[1]-v2[1])*(pt[0] - v2[0]) - (v0[0]-v2[0])*(pt[1] - v2[1])
}

/// Shape - collection of lines and points to render
pub struct Shape {
    pub vertices_original : Vertices,
    pub vertices: Vertices,
    pub color : Vec<i32>,
    pub style : Vec<i32>, // 0 point, 1 line to next vertex
}

impl Shape {
    fn reset (&mut self) {
        self.vertices = self.vertices_original.clone();
    }
}

/// Shape with motion
#[derive(PartialEq, Debug)]
pub enum EntityCast {
    PLAYER, BULLET, ENEMY, STAR
}
pub struct Entity {
    pub power: bool,  // is this alive/dead
    pub cast: EntityCast,
    pub age: i32,     // age in ticks
    pub shape: Shape, // The meat popsicle
    pub location: Vertex, // Cartesian location
    pub velocity: Vertex, // Where's its headed
    pub angle: f32,  // Current rotation
    pub vangle: f32, // Where it's rotating
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

    // Scale all entity's vertices (origin normalized) to viewport width x height
    fn wrap (
        &mut self,
        tb :&Tbuff
    ) -> &mut Self
    {
        if !self.power { return self; }
        if self.location[0] < -1.0 { self.location[0] += 2.0; }
        if self.location[0] > 1.0  { self.location[0] -= 2.0; }
        if self.location[1] < -1.0 { self.location[1] += 2.0; }
        if self.location[1] > 1.0  { self.location[1] -= 2.0; }
        return self;
    }
    // Scale all entity's vertices (origin normalized) to viewport width x height
    fn scale_to_viewport (
        &mut self,
        tb :&Tbuff,
        w:u32,
        h:u32
    ) -> &mut Self
    {
        if !self.power { return self; }
        translate(&mut self.shape.vertices, &[1.0, 1.0, 1.0, 0.0]);
        scale(&mut self.shape.vertices, &[w as f32 / 2.0, h as f32 / 2.0]);
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
            for [xinc, yinc, ch] in utils::Walk::new(&vs1[0..2], &vs2[0..2]) {
                x += xinc;
                y += yinc;
                tb.set(x as usize, y as usize, 0, 1, ch as u8 as char);
            }
        } // while
        self
    } // draw_spokes

    // Plot the vertices in the shape.  Two types supported:  lines and points
    fn plot_shape (
        self : &mut Entity,
        context :piston_window::Context,
        graphics :&mut G2d
    ) -> &mut Self{
        if !self.power { return self; }
        let shape = &self.shape;
        for v in 0..shape.vertices.len() {
            if shape.color[v] == 0 { continue }
            if 0 == shape.style[v] {
                rectangle(
                    [ 0.0, 1.0, 0.0, 1.0 ],
                    [ shape.vertices[v][0] as f64, shape.vertices[v][1] as f64,
                      1.0,                    1.0 ],
                    context.transform,
                    graphics);
            }  else if 1 == shape.style[v] {
                line(
                    [ 0.0, 1.0, 0.0, 0.20 ],
                    0.50,
                    [ shape.vertices[v][0] as f64, shape.vertices[v][1] as f64,
                      shape.vertices[v+1][0] as f64, shape.vertices[v+1][1] as f64],
                    context.transform,
                    graphics);
            } // else
        } // while

        // Emphasize line endpoints
        for v in 0..shape.vertices.len() {
            if shape.color[v] == 0 { continue }
            if 1 == shape.style[v] {
                rectangle(
                    [ 0.0, 1.0, 0.0, 0.5 ],
                    [ shape.vertices[v][0] as f64 - 0.5, shape.vertices[v][1] as f64 - 0.5,
                      1.0,                    1.0 ],
                    context.transform,
                    graphics);
            }
        } // while
        return self;
    } // plot_shape

    fn draw_shape (
            self : &mut Entity,
            tb  :&mut term::Tbuff
    ) -> &mut Self {
        if !self.power { return self; }
        let shape = &self.shape;
        for v in 0..shape.vertices.len() {
            if shape.color[v] == 0 { continue }
            if 0 == shape.style[v] {
                tb.set(
                     shape.vertices[v][0] as usize,
                     shape.vertices[v][1] as usize,
                     0, shape.color[v], '*');
            }  else if 1 == shape.style[v] {
               tb.line(&shape.vertices[v], &shape.vertices[v+1], '@', shape.color[v]);
            } // else
        } // while
        return self;
    } // draw_shape

} // impl Entity

/// Entities - Container of all entities
pub struct Entities {
   pub entities :Vec<Entity>
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

    pub fn tick_and_transform (&mut self) -> &mut Self {
          self.iter_mut().map(Entity::tick_and_transform).count();
          self
    }

    pub fn scale_to_terminal (&mut self, tb :&Tbuff) -> &mut Self {
          self.iter_mut().map( |e|e.scale_to_terminal(tb) ).count();
          self
    }

    pub fn wrap (&mut self, tb :&Tbuff) -> &mut Self {
          self.iter_mut().map( |e| e.wrap(tb) ).count();
          self
    }

    // Scale all origin-normalized objects to viewport
    pub fn scale_to_viewport (&mut self, tb :&Tbuff, w:u32, h:u32) -> &mut Self {
          self.iter_mut().map( |e|e.scale_to_viewport(tb, w, h) ).count();
          self
    }
    pub fn scale_to_terminal_origin_center (&mut self, tb :&Tbuff) -> &mut Self {
          let center_loc = self.entities[1].location.clone();
          self.iter_mut().enumerate().map( |(i,e)|e.scale_to_terminal_origin_center(tb,
            //if 1 == i { &[0.0, 0.0, 0.0, 0.0] } else { &center_loc }
            &[0.0, 0.0, 0.0, 0.0]
            //&center_loc
          )).count();
          self
    }

    // Hide bullet after a while (limited life span)
    pub fn expire_bullet (&mut self) -> &mut Self {
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

    pub fn draw_shapes (&mut self, tb :& mut term::Tbuff) {
        for e in self.iter_mut() {
            //e.draw_spokes(tb);
            e.draw_shape(tb);
        }
    }

    pub fn plot_shapes (
        &mut self,
        context :piston_window::Context,
        graphics :&mut G2d
    ) {
        for e in self.iter_mut() {
            e.plot_shape(context, graphics);
        }
    }
} // impl Entities

pub fn main () {
    println!("== main/src/lag.rs:main() {:?} ====", core::module_path!());
}