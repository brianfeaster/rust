#![allow(dead_code, unused_variables, non_snake_case)]

use ::std::sync::{Arc, Mutex};
use ::std::fmt;
use ::std::ops::{Add, Mul, AddAssign, MulAssign};
use ::std::time::{SystemTime};
use ::piston_window::{*};

#[derive(Debug)]
struct State { x:f64, y:f64, i:i32, j:i32 }

impl State {
    fn new() -> State { State{ x:0.0, y:0.0, i:0, j:0 } }
    fn i(&mut self) -> i32 { self.i += 1; self.i - 1 }
    fn j(&mut self) -> i32 { self.j += 1; self.j - 1 }
}

#[derive(Copy, Clone)]
struct M4 {
    a:f64, b:f64, c:f64, d:f64, 
    e:f64, f:f64, g:f64, h:f64, 
    i:f64, j:f64, k:f64, l:f64, 
    m:f64, n:f64, o:f64, p:f64
}

impl fmt::Debug for M4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:5.2} {:5.2} {:5.2} {:5.2}\n {:5.2} {:5.2} {:5.2} {:5.2}\n {:5.2} {:5.2} {:5.2} {:5.2}\n {:5.2} {:5.2} {:5.2} {:5.2}]",
         self.a, self.b, self.c, self.d,
         self.e, self.f, self.g, self.h,
         self.i, self.j, self.k, self.l,
         self.m, self.n, self.o, self.p)
    }
}

impl AddAssign<[f64; 3]> for M4 {
    fn add_assign(&mut self, t:[f64; 3]) {
        self.d += self.a*t[0] + self.b*t[1] + self.c*t[2];
        self.h += self.e*t[0] + self.f*t[1] + self.g*t[2];
        self.l += self.i*t[0] + self.j*t[1] + self.k*t[2];
        self.p += self.m*t[0] + self.n*t[1] + self.o*t[2];
    }
}
impl Add<[f64; 3]> for M4 {
    type Output = M4;
    fn add(mut self, t:[f64; 3]) -> M4 {
        self.d += self.a*t[0] + self.b*t[1] + self.c*t[2];
        self.h += self.e*t[0] + self.f*t[1] + self.g*t[2];
        self.l += self.i*t[0] + self.j*t[1] + self.k*t[2];
        self.p += self.m*t[0] + self.n*t[1] + self.o*t[2];
        self
    }
}

impl MulAssign<f64> for M4 {
    fn mul_assign(&mut self, s:f64) {
        self.a *= s;  self.b *= s;  self.c *= s;
        self.e *= s;  self.f *= s;  self.g *= s;
        self.i *= s;  self.j *= s;  self.k *= s;
        self.m *= s;  self.n *= s;  self.o *= s;
    }
}

impl Mul<f64> for M4 {
    type Output = M4;
    fn mul(mut self, s:f64) -> M4 {
        self.a *= s;  self.b *= s;  self.c *= s;
        self.e *= s;  self.f *= s;  self.g *= s;
        self.i *= s;  self.j *= s;  self.k *= s;
        self.m *= s;  self.n *= s;  self.o *= s;
        self
    }
}

impl Mul<M4> for M4 {
    type Output = M4;
    fn mul(self, b: M4) -> M4 {
        M4{
         a: self.a*b.a + self.b*b.e + self.c*b.i + self.d*b.m,
         b: self.a*b.b + self.b*b.f + self.c*b.j + self.d*b.n,
         c: self.a*b.c + self.b*b.g + self.c*b.k + self.d*b.o,
         d: self.a*b.d + self.b*b.h + self.c*b.l + self.d*b.p,
         
         e: self.e*b.a + self.f*b.e + self.g*b.i + self.h*b.m,
         f: self.e*b.b + self.f*b.f + self.g*b.j + self.h*b.n,
         g: self.e*b.c + self.f*b.g + self.g*b.k + self.h*b.o,
         h: self.e*b.d + self.f*b.h + self.g*b.l + self.h*b.p,

         i: self.i*b.a + self.j*b.e + self.k*b.i + self.l*b.m,
         j: self.i*b.b + self.j*b.f + self.k*b.j + self.l*b.n,
         k: self.i*b.c + self.j*b.g + self.k*b.k + self.l*b.o,
         l: self.i*b.d + self.j*b.h + self.k*b.l + self.l*b.p,

         m: self.m*b.a + self.n*b.e + self.o*b.i + self.p*b.m,
         n: self.m*b.b + self.n*b.f + self.o*b.j + self.p*b.n,
         o: self.m*b.c + self.n*b.g + self.o*b.k + self.p*b.o,
         p: self.m*b.d + self.n*b.h + self.o*b.l + self.p*b.p}
    }
}

type V4 = [f64; 4];

// Scale x, y , z
impl Mul<V4> for M4 {
    type Output = M4;
    fn mul(mut self, v:V4) -> Self {
        self.d += self.a*v[0] + self.b*v[1] + self.c*v[2];
        self.h += self.e*v[0] + self.f*v[1] + self.g*v[2];
        self.l += self.i*v[0] + self.j*v[1] + self.k*v[2];
        self.p += self.m*v[0] + self.n*v[1] + self.o*v[2];
        self
    }
}

enum Rot { RotX(f64), RotY(f64), RotZ(f64) }

//Rot Z
impl MulAssign<Rot> for M4 {
    fn mul_assign(&mut self, s:Rot) {
        match s {
            Rot::RotX(f) => {
                let c = f.cos();
                let s = f.sin();
                let mb = self.b; // abcd 1...
                let mf = self.f; // efgh .CZ.
                let mj = self.j; // ijkl .SC.
                let mn = self.n; // mnop ...1
                self.b = mb*c - self.c*s;  self.c = mb*s + self.c*c;
                self.f = mf*c - self.g*s;  self.g = mf*s + self.g*c;
                self.j = mj*c - self.k*s;  self.k = mj*s + self.k*c;
                self.n = mn*c - self.o*s;  self.o = mn*s + self.o*c;
            },
            Rot::RotY(f) => {
                let c = f.cos();
                let s = f.sin();
                let ma = self.a; // abcd   C.S.
                let me = self.e; // efgh * .1..
                let mi = self.i; // ijkl   Z.C.
                let mm = self.m; // mnop   ...1
                self.a = ma*c - self.c*s;  self.c = ma*s + self.c*c;
                self.e = me*c - self.g*s;  self.g = me*s + self.g*c;
                self.i = mi*c - self.k*s;  self.k = mi*s + self.k*c;
                self.m = mm*c - self.o*s;  self.o = mm*s + self.o*c;
            },
            Rot::RotZ(f) => {
                let s = f.sin();
                let c = f.cos();
                let ma = self.a;
                let me = self.e;
                let mi = self.i;
                let mm = self.m;
                self.a = ma*c + self.b*s;  self.b = -ma*s + self.b*c;
                self.e = me*c + self.f*s;  self.f = -me*s + self.f*c;
                self.i = mi*c + self.j*s;  self.j = -mi*s + self.j*c;
                self.m = mm*c + self.n*s;  self.n = -mm*s + self.n*c;
            }
        } // match
    } // fn mul_assign
} // impl MulAssign


const M4_ID :M4 =
    M4{
        a:1.0, b:0.0, c:0.0, d:0.0,
        e:0.0, f:1.0, g:0.0, h:0.0,
        i:0.0, j:0.0, k:1.0, l:0.0,
        m:0.0, n:0.0, o:0.0, p:1.0
    };

const V4_ID :V4 = [0.0,0.0,0.0,1.0];

/*
fn scale (m :&mut M4, s :f64) -> &mut M4 {
    m[0] *= s;   m[1]  *= s;  m[2]  *= s;
    m[4] *= s;   m[5]  *= s;  m[6]  *= s;
    m[8] *= s;   m[9]  *= s;  m[10] *= s;
    //m[12] *= s;  m[13] *= s;  m[14] *= s;
    m
}
*/

fn trans (m :&mut M4, x :f64, y :f64, z :f64) -> &mut M4 {
    m.d += m.a*x + m.b*y + m.c*z;
    m.h += m.e*x + m.f*y + m.g*z;
    m.l += m.i*x + m.j*y + m.k*z;
    //m.p += m[12]*x + m[13]*y + m[14]*z;
    m
}

fn transPost (m :&mut M4, x :f64, y :f64, z :f64) -> &mut M4 {
    m.a += x*m.m;  m.b += x*m.n;  m.c += x*m.o;  m.d += x*m.p;
    m.e += y*m.m;  m.f += y*m.n;  m.g += y*m.o;  m.h += y*m.p;
    m.i += z*m.m;  m.j += z*m.n;  m.k += x*m.o;  m.l += z*m.p;
    m
}

fn rotz (m :&mut M4, ang :f64) -> &mut M4 {
    let s = ang.sin();
    let c = ang.cos();
    let m0 = m.a;
    let m4 = m.e;
    let m8 = m.i;
    let m13 = m.n;
    m.a = m0*c  + m.b*s;  m.b = -m0*s  + m.b*c;
    m.e = m4*c  + m.f*s;  m.f = -m4*s  + m.f*c;
    m.i = m8*c  + m.j*s;  m.j = -m8*s  + m.j*c;
    m.m = m13*c + m.o*s;  m.o = -m13*s + m.o*c;
    m
}

fn rotzPost (m :&mut M4, ang :f64) -> &mut M4 {
    let c = ang.cos();
    let s = ang.sin();
    let ma = m.a; let mb = m.b; let mc = m.c; let md = m.d;
    let me = m.e; let mf = m.f; let mg = m.g; let mh = m.h;
    m.a =  s*ma+c*me;  m.b =  s*mb+c*mf; m.c =  s*mc+c*mg; m.d =  s*md+c*mh;
    m.e = -c*ma+s*me;  m.f = -c*mb+s*mf; m.g = -c*mc+s*mg; m.h = -c*md+s*mh;
    m
}

fn roty (m :&mut M4, ang :f64) -> &mut M4 {
    let c = ang.cos();
    let s = ang.sin();
    let m0 = m.a;
    let m4 = m.e;
    let m8 = m.i;
    let m12 = m.m;
    //let m12 = m.m;
    m.a = m0*c - m.c*s;  m.c = m0*s + m.c*c;
    m.e = m4*c - m.g*s;  m.g = m4*s + m.g*c;
    m.i = m8*c - m.k*s;  m.k = m8*s + m.k*c;
    m.m = m12*c- m.o*s;  m.o = m12*s+ m.o*c;
    m
}

fn rotyPost (m :&mut M4, ang :f64) -> &mut M4 {
    let c = ang.cos();
    let s = ang.sin();
    let ma = m.a; let mb = m.b; let mc = m.c; let md = m.d;
    let mi = m.i; let mj = m.j; let mk = m.k; let ml = m.l;
    m.a =  c*ma+s*mi;  m.b =  c*mb+s*mj; m.c =  c*mc+s*mk; m.d =  c*md+s*ml;
    m.i = -s*ma+c*mi;  m.j = -s*mb+c*mj; m.k = -s*mc+c*mk; m.l = -s*md+c*ml;
    m
}

fn rotx (m :&mut M4, ang :f64) -> &mut M4 {
    let c = ang.cos();
    let s = ang.sin();
    let m1  = m.b;
    let m5  = m.f;
    let m9  = m.j;
    let m13 = m.n;
    m.b = m1*c + m.c*s;  m.c = -m1*s + m.c*c;
    m.f = m5*c + m.g*s;  m.g = -m5*s + m.g*c;
    m.j = m9*c + m.k*s;  m.k = -m9*s + m.k*c;
    m.n = m13*c+ m.o*s;  m.o =-m13*s + m.o*c;
    m
}

fn rotxPost (m :&mut M4, ang :f64) -> &mut M4 {
    let c = ang.cos();
    let s = ang.sin();
    let me = m.e; let mf = m.f; let mg = m.g; let mh = m.h;
    let mi = m.i; let mj = m.j; let mk = m.k; let ml = m.l;
    m.e =  c*me+s*mi;  m.e =  c*me+s*mj; m.g =  c*mg+s*mk; m.h =  c*mh+s*ml;
    m.i = -s*me+c*mi;  m.j = -s*mf+c*mj; m.k = -s*mg+c*mk; m.l = -s*mh+c*ml;
    m
}

fn persPost (m :&mut M4) -> &mut M4 {
    m.m=m.i;  m.n=m.j;  m.o=m.k;  m.p=m.l;
    m
}

fn xformmut(m: &M4,  v: &mut V4) {
    let a=v[0]; let b=v[1]; let c=v[2]; let d=v[3];
    v[0] = m.a*a  + m.b*b  + m.c*c  + m.d*d;
    v[1] = m.e*a  + m.f*b  + m.g*c  + m.h*d;
    v[2] = m.i*a  + m.j*b  + m.k*c  + m.l*d;
    v[3] = m.m*a  + m.n*b  + m.o*c  + m.p*d;
}

fn xform (m: &M4, v: &V4) -> [f64; 2] {
    let a=v[0]; let b=v[1]; let c=v[2]; let d=v[3];
    let x = m.a*a  + m.b*b  + m.c*c  + m.d*d;
    let y = m.e*a  + m.f*b  + m.g*c  + m.h*d;
    let z = (m.i*a  + m.j*b  + m.k*c  + m.l*d) * 0.004;
    //let n = m.m*a  + m.n*b  + m.o*c  + m.p*d;
    if z < 0.0 {
        [::std::f64::NAN, ::std::f64::NAN]
    } else {
        [x/z, y/z]
    }
}

// Ornament
struct Orn {
    poly: Vec<V4>,
    mat: M4,
    c: [f32; 4],
    update: bool
}

fn fun_piston() {
    const W :f64 = 1200.0;
    const H :f64 = 600.0;
    let mut state :State = State::new();
    let bs = 55.0_f64;

    let window_settings =
        ::piston::window::WindowSettings::new("ASCIIRhOIDS", [256 as u32, 256 as u32]);
    let mut pwin : ::glutin_window::GlutinWindow =
        window_settings
            .graphics_api(::opengl_graphics::OpenGL::V3_2)
            .exit_on_esc(true)
            .size(piston_window::Size{width :W, height :H})
            .decorated(true)
            .build()
            .unwrap();

    let mut polys = vec![];
    let mut y = 0.0;

    for i in 0..1000 {
        let mut mat = M4_ID;
        y += 0.1 - crate::r64(i as f32) / 50000.0;
        mat *= Rot::RotY(crate::r64(6.28));
        mat += [0.0,  (50.0 - y)/100.0,  (2.0+i as f64/40.0)/100.0];
        mat *=  0.002 ;
        polys.push(
            Orn {
                poly: vec![
                    [-1.0, -1.0,  0.0, 1.0],
                    [ 1.0, -1.0,  0.0, 1.0],
                    [ 1.0,  1.0,  0.0, 1.0],
                    [-1.0,  1.0,  0.0, 1.0]],
                mat: mat,
                c: [crate::r32(1.0), crate::r32(1.0), crate::r32(1.0), 0.5],
                update: true
            } // Orn
        );
    }

    polys.push(
        Orn {
            poly: vec![
                [-1.0, 0.0, -1.0, 1.0],
                [ 1.0, 0.0, -1.0, 1.0],
                [ 1.0, 0.0,  1.0, 1.0],
                [-1.0, 0.0,  1.0, 1.0]],
            mat: (M4_ID * 0.5) + [0.0, -1.0, 0.0],
            c: [0.0, 0.5, 0.0, 0.1],
            update: false
        });

    let epoch :SystemTime = SystemTime::now(); // For FPS calculation
    let mut mx = 0.0_f64;
    let mut my = 0.0_f64;
    let mut doit = true;
    let mut gl : ::opengl_graphics::GlGraphics =
        ::opengl_graphics::GlGraphics::new(::opengl_graphics::OpenGL::V3_2);

    let mut events = ::piston::event_loop::Events::new(EventSettings::new());

    // Apply each object's xform to itself once
    /*
    for poly in polys.iter_mut() {
        for vi in 0..poly.poly.len() {
            xformmut(&poly.mat, &mut poly.poly[vi]);
        }
    }
    */

    while let Some(event) = events.next(&mut pwin) {

        /*
        match &event { //::piston_window::Event
           Event::Input(i, j)   => println!("\x1b[1;31mInput {:?} {:?}", i, j),
           Event::Loop(Loop::Idle(::piston::input::IdleArgs{dt})) => println!("\x1b[0;32mLoop/Idle dt={:?}", dt),
           Event::Loop(i)       => println!("\x1b[1;32mLoop {:?}", i),
           Event::Custom(i,j,k) => println!("\x1b[1;34mCustom {:?} {:?} {:?}", i, j, k),
        }
        */

        //println!("\x1b[31m{:?} \x1b[0m", event);
        if event.idle_args() != None {
            //println!("{:?} {}", event, 1000.0 * state.i as f32 / epoch.elapsed().unwrap().as_millis() as f32);
            //break
        }
        if event.resize_args() != None { }
        if let Some([x,y]) = event.mouse_cursor_args() {
           mx = x - W/2.0;
           my = y - H/2.0;
           doit = true;
           //print!("\x1b[33m{:.1},{:.1} \x1b[0m", x, y);
           ::util::flush();
        }
        if let Some(args) = event.button_args() {
            match args.button {
                Button::Keyboard(Key::Q) => pwin.set_should_close(true),
                Button::Keyboard(Key::S) => { state.x += -0.01 * (mx*0.01).sin();  state.y += -0.01 * (mx*0.01).cos(); },
                Button::Keyboard(Key::A) => { state.x +=  0.01 * (mx*0.01).sin();  state.y +=  0.01 * (mx*0.01).cos(); },
                Button::Keyboard(Key::D) => { state.x +=  0.01 * (mx*0.01+1.57).sin(); state.y +=  0.01 * (mx*0.01+1.57).cos(); },
                Button::Keyboard(Key::F) => { state.x +=  0.01 * (mx*0.01-1.57).sin(); state.y +=  0.01 * (mx*0.01-1.57).cos(); },
                w => ()
            }
        }
        if let Some(args) = event.text_args() {
            //println!("text_args == {:?}", args);
            if args == " " { ::util::sleep(500) }
        }
        if doit && event.render_args() != None {
            let i = state.i();
            let args = event.render_args().unwrap();

            let mut gmat = M4_ID;
            rotx(&mut gmat, -my*0.01);
            roty(&mut gmat, -mx*0.01);
            trans(&mut gmat, state.x, 0.2, 1.0+state.y);
            //gmat *= Rot::RotY(i as f64 / 50.0);

            if i % 1000 == 0 {
                print!("\x1b[0;32m{:.1}\x1b[0m ", 1000.0 * state.i as f32 / epoch.elapsed().unwrap().as_millis() as f32);
                ::util::flush();
            }


            gl.draw( args.viewport(), |context :graphics::Context, graphics| {
                //persPost(&mut gmat);
                ::graphics::clear([0.0, 0.0, 0.0, 1.0], graphics);
                let mut ii = 1f64;
                for poly in polys.iter_mut() {
                    ii += 0.01;
                    let mut mat = poly.mat;
                    if poly.update {
                        //mat *= Rot::RotZ(i as f64 / 10.0); // Mutate object's transform
                        //mat *= Rot::RotX(i as f64 / 120.0); // Mutate object's transform
                        mat *= Rot::RotY(i as f64 / 100.0 + ii); // Mutate object's transform
                        mat *= 2f64; //(i as f64 / 10.0 + ii).sin() * 1.0 + 1.5;
                    }
                    let tmat = gmat * mat; // Combine camera and obj transofmr

                    // Transform all the polygons
                    let polys = poly.poly
                        .iter()
                        .map( |v| xform(&tmat, &v) )
                        .filter( |v| ::std::f64::NAN != v[0])
                        .collect::<Vec<[f64; 2]>>();

                    if 2 < polys.len()  {
                        ::graphics::polygon(
                            poly.c,
                            &polys,
                            [[0.01, 0.0, 0.0], [0.0, 0.01, 0.0]], //context.transform,
                            graphics)
                    } // if
                } // for poly
            }); // lambda // draw_2
            //::util::sleep(100);
            //doit = false;
        } // if render_args
        else if event.update_args() != None { }
        else if event.after_render_args() != None { }
        //else { println!("{:?}", event); }
    } // while
}

////////////////////////////////////////////////////////////////////////////////
pub fn main() {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    fun_piston();
}

/* NOTES
 rotZ   mat
23..   abcd    2a+3e 2b+3f 2c+3g 2d+3h
42.. * efgh =  4a+2e 4b+2f 4c+2g 4d+2h
..1.   ijkl    i     j     k     l
...1   mnop    m     n     o     p

 xlate  mat
1..x   abcd    a+xm b+xn c+xo d+xp
.1.y * efgh =  e+ym f+yn g+yo h+yp
..1z   ijkl    i+zm j+zn k+zo l+zp
...1   mnop    m    n    o    p

 xlate  mat
abcd   1..x    a    b    c    ax+by+cz+d
efgh * .1.y =  e    f    g    ex+fy+gz+h
ijkl   ..1z    i    j    k    ix+jy+kz+l
mnop   ...1    m    n    o    mx+ny+oz+p

 mat    scaleXYZ
abcd   2... = a2 b3 c4 d
efgh * .3..   e2 f3 g4 h
ijkl   ..4.   i2 j3 k4 l
mnop   ...1   m2 n3 o4 p

 mat    addXYZ
abcd   2... = a2 b3 c4 d
efgh * .3..   e2 f3 g4 h
ijkl   ..4.   i2 j3 k4 l
mnop   ...1   m2 n3 o4 p


 mat         rotZ
a b c d   2 3 . .    a2+b4 a3+b2 c     d
e f g h * 4 2 . . =  e2+f4 e3+f2 g     h
i j k l   . . 1 .    i2+j4 i3+j2 k     l
m n o p   . . . 1    m2+n4 m3+n2 o     p

roty  mat
c.s. abcd ca+Si cb+sj cc+sk cd+sl
.1.. efgh e     f       g      h
S.c. ijkl Sa+ci Sb+cj Sc+ck Sd+cl
...1 mnop m     n       o      p

 mat   roty   c=cos s=sin S = -sin
abcd   c.s.   ac+cS b as+cc d
efgh * .1.. = ec+gS f es+gc h
ijkl   S.c.   ic+kS j is+kc l
mnop   ...1   mc+oS n ms+oc p

 mat  roty
 abcd c.s.  ac+cS b as+cc d 
 efgh .1..  ec+gS f es+gc h
 ijkl S.c.  ic+kS j is+kc l
 mnop ...1  mc+oS n ms+oc p

 mat   rotx   c=cos s=sin S = -sin
abcd   1...   a bc+cs bS+cc d
efgh * .cS. = e fc+gs fS+gc h
ijkl   .sc.   i jc+ks jS+kc l
mnop   ...1   m nc_os nS+oc p

rotZ  mat
st.. abcd  sa+te sb+tf sc+tg sd+th
Ts.. efgh  Ta+se Tb+sF Tc+sg Td+sh
..1. ijkl  i     j     k     l
...1 mnop  m     n     o     p

mat   xlat
abcd 1..x   a b c ax+by+cz+d
efgh*.1.y = e f g ex+fy+gz+h
ijkl ..1z   i j k ix+jy+kz+l
mnop ...1   m n o mx+ny+oz+p

persp    mat   
1...   abcd   a b c d 
.1.. * efgh = e f g h 
..1.   ijkl   i j k l
..1.   mnop   i j k l 
                    println!("\x1b[7{:?}", &mat[0..4]);
                    println!("{:?}", &mat[4..8]);
                    println!("{:?}", &mat[8..12]);
                    println!("{:?}\x1b[8", &mat[12..16]);

xlate  mat
1..x  abcd  a+xm b+xn c+xo d+xp
.1.y  efgh  e+ym f+yn g+yo h+yp
..1z  ijkl  i+zm j+zn k+zo l+zp
...1  mnop  m    n    o    p
*/