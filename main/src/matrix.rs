use ::std::fmt;
use ::std::ops::{Add, Mul, AddAssign, MulAssign};
use ::std::time::{SystemTime};

use ::graphics::{Graphics, DrawState};
//use ::opengl_graphics::{GlGraphics, OpenGL, Colored, Textured, TexturedColor};
use ::opengl_graphics::{GlGraphics, OpenGL, Colored, Textured};
use ::piston::*;


use ::glutin_window::{GlutinWindow};

use ::life::*;

const CF2 :&str = "\x1b[32m";

#[derive(Debug)]
struct State {
    W:f64, H:f64,
    x:f64, y:f64, z:f64,
    mx: f64, my: f64,
    tick: u64,
    epoch: SystemTime
}

impl State {
    fn new() -> State {
        State{
            W:1400.0, H:700.0, // window
            x:0.0, y:0.0, z:0.0, // player
            mx:0.0, my:0.0, // mouse
            tick:0,
            epoch:SystemTime::now(),
         }
    } // new()
    fn tick(&mut self) -> &mut Self { self.tick += 1; self }
    fn printfps(&self, doit: bool) {
        if doit && 0 == self.tick % 50 {
            print!("{}{} ", CF2, self.tick / self.epoch.elapsed().unwrap().as_secs());
            ::util::flush();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Enter the matrix

// Homogeneous Vertex
type V4 = [f64; 4];

/// Transformation Matrix
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

////////////////////////////////////////////////////////////////////////////////
// Regular math (not mut)

/// M4 * V4 -> M4'
impl Mul<M4> for M4 {
    type Output = M4;
    fn mul(mut self, t: M4) -> Self {
        let a=self.a; let b=self.b; let c=self.c; let d=self.d;
        self.a = a*t.a + b*t.e + c*t.i + d*t.m;
        self.b = a*t.b + b*t.f + c*t.j + d*t.n;
        self.c = a*t.c + b*t.g + c*t.k + d*t.o;
        self.d = a*t.d + b*t.h + c*t.l + d*t.p;

        let e=self.e; let f=self.f; let g=self.g; let h=self.h;
        self.e = e*t.a + f*t.e + g*t.i + h*t.m;
        self.f = e*t.b + f*t.f + g*t.j + h*t.n;
        self.g = e*t.c + f*t.g + g*t.k + h*t.o;
        self.h = e*t.d + f*t.h + g*t.l + h*t.p;

        let i=self.i; let j=self.j; let k=self.k; let l=self.l;
        self.i = i*t.a + j*t.e + k*t.i + l*t.m;
        self.j = i*t.b + j*t.f + k*t.j + l*t.n;
        self.k = i*t.c + j*t.g + k*t.k + l*t.o;
        self.l = i*t.d + j*t.h + k*t.l + l*t.p;

        let m=self.m; let n=self.n; let o=self.o; let p=self.p;
        self.m = m*t.a + n*t.e + o*t.i + p*t.m;
        self.n = m*t.b + n*t.f + o*t.j + p*t.n;
        self.o = m*t.c + n*t.g + o*t.k + p*t.o;
        self.p = m*t.d + n*t.h + o*t.l + p*t.p;
        self
    }
}
/// M4 * V4 -> V4'
/// abcd   X   aX + bY + cZ + dW
/// efgh * Y = eX + fY + gZ + hW
/// ijkl   Z   iX + jY + kZ + lW
/// mnop   W   mX + nY + oZ + pW
impl Mul<V4> for M4 {
    type Output = M4;
    fn mul(mut self, v:V4) -> Self {
        self.d = self.a*v[0] + self.b*v[1] + self.c*v[2] + self.d*v[3];
        self.h = self.e*v[0] + self.f*v[1] + self.g*v[2] + self.h*v[3];
        self.l = self.i*v[0] + self.j*v[1] + self.k*v[2] + self.l*v[3];
        self.p = self.m*v[0] + self.n*v[1] + self.o*v[2] + self.p*v[3];
        self
    }
}


////////////////////////////////////////////////////////////////////////////////
// Scale matrix   M4 * M4' -> M4''
//      SCALE
// abcd  X...   aX bY cZ d
// efgh *.Y.. = eX fY gZ h
// ijkl  ..Z.   iX jY kZ l
// mnop  ...1   mX nY oZ p

impl Mul<f64> for M4 {
    type Output = M4;
    fn mul(mut self, s:f64) -> Self {
        self.a *= s;  self.b *= s;  self.c *= s;
        self.e *= s;  self.f *= s;  self.g *= s;
        self.i *= s;  self.j *= s;  self.k *= s;
        self.m *= s;  self.n *= s;  self.o *= s;
        self
    }
}

impl Mul<[f64; 3]> for M4 {
    type Output = M4;
    fn mul(mut self, s:[f64; 3]) -> Self {
        self.a *= s[0];  self.b *= s[1];  self.c *= s[2];
        self.e *= s[0];  self.f *= s[1];  self.g *= s[2];
        self.i *= s[0];  self.j *= s[1];  self.k *= s[2];
        self.m *= s[0];  self.n *= s[1];  self.o *= s[2];
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

impl MulAssign<[f64;3]> for M4 {
    fn mul_assign(&mut self, s:[f64;3]) {
        self.a *= s[0];  self.b *= s[1];  self.c *= s[2];
        self.e *= s[0];  self.f *= s[1];  self.g *= s[2];
        self.i *= s[0];  self.j *= s[1];  self.k *= s[2];
        self.m *= s[0];  self.n *= s[1];  self.o *= s[2];
    }
}

////////////////////////////////////////////////////////////////////////////////
// SCALE POST

// X...   abcd   Xa Xb Xc Xd
// .Y.. * efgh = Ye Yf Yg Yh
// ..Z.   ijkl   Zi Zj Zk Zl
// ...1   mnop   m  n  o  p

fn scalePost(m: &mut M4, s:[f64; 3]) {
    m.a *= s[0];  m.b *= s[0];  m.c *= s[0];  m.d *= s[0];
    m.e *= s[1];  m.f *= s[1];  m.g *= s[1];  m.h *= s[1];
    m.i *= s[2];  m.j *= s[2];  m.k *= s[2];  m.l *= s[2];
}

////////////////////////////////////////////////////////////////////////////////

enum Rot {
    // Pre rotation
    RotX(f64),
    RotY(f64),
    RotZ(f64)
    // Post rotation???
}

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

////////////////////////////////////////////////////////////////////////////////
// Transform vector with matrix

/// xformmut(M4, !V4)
fn xformmut(m: &M4,  v: &mut V4) {
    let a=v[0]; let b=v[1]; let c=v[2]; let d=v[3];
    v[0] = m.a*a  + m.b*b  + m.c*c  + m.d*d;
    v[1] = m.e*a  + m.f*b  + m.g*c  + m.h*d;
    v[2] = m.i*a  + m.j*b  + m.k*c  + m.l*d;
    v[3] = m.m*a  + m.n*b  + m.o*c  + m.p*d;
}

/// M4 *= V4
impl MulAssign<M4> for V4 {
    fn mul_assign(&mut self, m:M4) {
        let a=self[0]; let b=self[1]; let c=self[2]; let d=self[3];
        self[0] = m.a*a  + m.b*b  + m.c*c  + m.d*d;
        self[1] = m.e*a  + m.f*b  + m.g*c  + m.h*d;
        self[2] = m.i*a  + m.j*b  + m.k*c  + m.l*d;
        self[3] = m.m*a  + m.n*b  + m.o*c  + m.p*d;
    }
}

////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////
//      TRANSLATE
// abcd   1..x   a b c ax+by+cz+d
// efgh * .1.y = e f g ex+fy+gz+h
// ijkl   ..1z   i j k ix+jy+kz+l
// mnop   ...1   m n o mx+ny+oz+p

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
// mnop   ...1   m n o mx+ny+oz+p
impl AddAssign<[f64; 3]> for M4 {
    fn add_assign(&mut self, [x,y,z]:[f64; 3]) {
        self.d += self.a*x + self.b*y + self.c*z;
        self.h += self.e*x + self.f*y + self.g*z;
        self.l += self.i*x + self.j*y + self.k*z;
        self.p += self.m*x + self.n*y + self.o*z;
    }
}

fn trans (m :&mut M4, x :f64, y :f64, z :f64) {
    m.d += m.a*x + m.b*y + m.c*z;
    m.h += m.e*x + m.f*y + m.g*z;
    m.l += m.i*x + m.j*y + m.k*z;
    m.p += m.m*x + m.n*y + m.o*z;
}

////////////////////////////////////////////////////////////////////////////////
// TRANSLATE POST
// 1..x   abcd   a+xm b+xn c+xo d+xp
// .1.y * efgh = e+ym f+yn g+yo h+yp
// ..1z   ijkl   i+zm j+zn k+zo l+zp
// ...1   mnop   m    n    o    p
fn transPost (m :&mut M4, x :f64, y :f64, z :f64) -> &mut M4 {
    m.a += x*m.m;  m.b += x*m.n;  m.c += x*m.o;  m.d += x*m.p;
    m.e += y*m.m;  m.f += y*m.n;  m.g += y*m.o;  m.h += y*m.p;
    m.i += z*m.m;  m.j += z*m.n;  m.k += x*m.o;  m.l += z*m.p;
    m
}

////////////////////////////////////////////////////////////////////////////////
// ROTATION

/*
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


fn rotx (
    m: &mut M4,
    ang: f64
) {
    let [c, s] = [ang.cos(), ang.sin()];
    let mb = m.b;
    let mf = m.f;
    let mj = m.j;
    let mn = m.n;
    m.b = mb*c + m.c*s;  m.c = -mb*s + m.c*c;
    m.f = mf*c + m.g*s;  m.g = -mf*s + m.g*c;
    m.j = mj*c + m.k*s;  m.k = -mj*s + m.k*c;
    m.n = mn*c + m.o*s;  m.o = -mn*s + m.o*c;
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
*/

fn xformperspectivecull (
    m: &M4,
    v: &V4
) -> [f64; 2] {
    let z = m.i*v[0] + m.j*v[1] + m.k*v[2] + m.l*v[3];
    if z <= 0.0 {
        [::std::f64::NAN, ::std::f64::NAN]
    } else {
        let x = m.a*v[0] + m.b*v[1] + m.c*v[2] + m.d*v[3];
        let y = m.e*v[0] + m.f*v[1] + m.g*v[2] + m.h*v[3];
        //let n = m.m*v[0] + m.n*v[1] + m.o*v[2] + m.p*v[3];
        [x/z, y/z]
    }
}

// Ornaments ///////////////////////////////////////////////////////////////////

#[derive (Debug)]
struct Orn {
    poly: Vec<V4>,
    mat: M4,
    c: [f32; 4],
    update: bool,
    alive: bool
}

fn make_polys() -> Vec<Orn> {
    let mut polys: Vec<Orn> = vec!();

    // Floor
    
    for y in 0..10 {
        for x in 0..10 {
            polys.push(
                Orn {
                    poly: vec![
                        [-1.0, 0.0, -1.0, 1.0],
                        [ 1.0, 0.0, -1.0, 1.0],
                        [ 1.0, 0.0,  1.0, 1.0],
                        [-1.0, 0.0,  1.0, 1.0]],
                    mat: (M4_ID * 0.5) + [-9.0 + (2*x) as f64, -1.0, -9.0 + (2*y) as f64],
                    c: [crate::r32(0.2)+0.2, 0.0, 0.0, 1.0],
                    update: false,
                    alive: true
                });
        }
    }
    for y in 0..10 {
        for x in 0..10 {
            polys.push(
                Orn {
                    poly: vec![
                        [-1.0, 0.0, -1.0, 1.0],
                        [ 1.0, 0.0, -1.0, 1.0],
                        [ 1.0, 0.0,  1.0, 1.0],
                        [-1.0, 0.0,  1.0, 1.0]],
                    mat: (M4_ID * 0.5) + [-9.0 + (2*x) as f64, 5.0, -9.0 + (2*y) as f64],
                    c: [crate::r32(0.2)+0.2, 0.0, 0.0, 1.0],
                    update: false,
                    alive: true
                });
        }
    }
    

    // Game of life cylinder
    for y in 0..50 {
      for x in 0..200 {
        let mut mat = M4_ID;
        mat *= Rot::RotY(6.28 * x as f64 / 200.0);
        mat += [0.0, y as f64 / 20.0 - 0.5, 1.0];
        mat *= [0.01, 0.02, 0.00];
        polys.push(
            Orn {
                poly: vec![
                    [-1.0, -1.0,  0.0, 1.0],
                    [ 1.0, -1.0,  0.0, 1.0],
                    [ 1.0,  1.0,  0.0, 1.0],
                    [-1.0,  1.0,  0.0, 1.0]],
                mat: mat,
                c: [crate::r32(1.0), crate::r32(1.0), crate::r32(1.0), 0.5],
                update: false,
                alive: true
            } // Orn
        );
      } // x
    } // y

    /*
    let mut y = 0.0; // height of square to play square along the cone
    for i in 0..1000 {
        let mut mat = M4_ID;
        y += 0.1 - crate::r64(i as f32) / 50000.0;
        mat *= Rot::RotY(crate::r64(6.28));
        mat += [0.0,  -y/100.0,  (2.0+i as f64/40.0)/100.0];
        mat *= 0.005;
        scalePost(&mut mat, [0.2, 0.2, 0.2]);
        polys.push(
            Orn {
                poly: vec![
                    [-1.0, -1.0,  0.0, 1.0],
                    [ 1.0, -1.0,  0.0, 1.0],
                    [ 1.0,  1.0,  0.0, 1.0],
                    [-1.0,  1.0,  0.0, 1.0]],
                mat: mat,
                c: [crate::r32(1.0), crate::r32(1.0), crate::r32(1.0), 0.5],
                update: true,
                alive: true
            } // Orn
        );
    }
    */


    polys
}

// Render //////////////////////////////////////////////////////////////////////

fn render_polygons (
    drawstate: &DrawState, // the global transform
    gfx: &mut GlGraphics,
    state: &mut State,
    polys: &mut Vec<Orn>,
    offset: f64,
    dbuff: Option<&Dbuff>
) {
    let i = state.tick as f64; // Global parameters for animation
    let mut ii = 0.0f64; // Local counter for animation

    // New global transform matrix, order matters.
    let mut gmat = M4_ID; // * [1.0, 1.0, 1.0]; // Must scale for perspective
    gmat *= Rot::RotX(state.my); // Must roate camera direction
    gmat *= Rot::RotY(-state.mx);
    gmat += [state.x, state.z, state.y]; // Must move camera location

    gmat += [0.0, 0.0, offset]; // Translate everything in scene
    //gmat *= Rot::RotY(i / 50.0); // Spin everything in scene around it's y-axis origin

    gfx.clear_color([0.0, 0.0, 0.0, 1.0]);
    gfx.clear_stencil(0);

    // Determine which Game Of Life cells are visible
    let goloffset = 200;
    if let Some(dbuff) = dbuff {
        let (dbuffa, dbuffb) = dbuff.buffs();
        for (i, e) in dbuffa.iter().zip(dbuffb.iter()).enumerate() {
        if *e.0 != *e.1 {
            if *e.0 == 0 {  // Died
                polys[i+goloffset].c[0] = 0.1;
                polys[i+goloffset].c[1] = 0.1;
                polys[i+goloffset].c[2] = 0.1;
            } else { // Born
                polys[i+goloffset].c[0] = crate::r32(1.0);
                polys[i+goloffset].c[1] = crate::r32(1.0);
                polys[i+goloffset].c[2] = crate::r32(1.0);
            }
        } }
    } // for in dbuff // if

    for poly in polys.iter_mut() { if poly.alive {
        let mut mat :M4 = gmat * poly.mat; // Create new transform matrix from global * object
        if poly.update {
            mat *= Rot::RotZ(i / 10.0); // Mutate object's transform
            mat *= Rot::RotX(i / 120.0); // Mutate object's transform
            mat *= Rot::RotY(i / 100.0 + ii); // Mutate object's transform
            mat *= 2f64; //(i as f64 / 10.0 + ii).sin() * 1.0 + 1.5;
        }
        ii += 0.01;

        // Transform and cull this polygon
        let polys = poly.poly
            .iter()
            .map( |v| xformperspectivecull(&mat, &v) )
            .filter( |v| ::std::f64::NAN != v[0])
            .map( |i| [i[0] as f32, i[1] as f32])
            .collect::<Vec<[f32; 2]>>();

        gfx.tri_list( drawstate, &poly.c, |f| (f)( 
            &[ [polys[0][0], polys[0][1]],
               [polys[1][0], polys[1][1]],
               [polys[2][0], polys[2][1]],
               
               [polys[0][0], polys[0][1]],
               [polys[2][0], polys[2][1]],
               [polys[3][0], polys[3][1]]]
        ))
    } } // if poly.alive // for poly
} // fn render

// REPL ////////////////////////////////////////////////////////////////////////

fn fun_piston() -> Result<usize, Box<dyn ::std::error::Error>>{
    let mut state: State = State::new();

    let mut polys = make_polys();
    let mut life = Life::new(200, 50);
    //life.clear();

    let ver = OpenGL::V3_2;

    let mut pwin =
        GlutinWindow::new(
            &WindowSettings::new( "ASCIIRhOIDS", [state.W as u32, state.H as u32] )
                .graphics_api(ver)
                .exit_on_esc(true)
                .size(piston_window::Size{width: state.W, height: state.H})
                .decorated(true)
        ).unwrap();

    let mut events = Events::new( EventSettings::new().max_fps(180) );

    //let glsl = ver.to_glsl();
    //let colored = Colored::new(glsl);
    //let textured = Textured::new(glsl);
    //let texturedcolor = TexturedColor::new(glsl);
    //let mut glgfx = GlGraphics::from_pieces(colored, textured, texturedcolor);
    let mut glgfx = GlGraphics::new(ver);

    life.gen_next();

    while let Some(event) = events.next(&mut pwin) { match event {
        Event::Loop( Loop::Render(args) ) => {
            if life.tick % 15 == 0 { life.add_glider(0, 0); }
            //if life.tick % 100 == 0 { life.randomize(s); }

            life.arena_xfer_dbuff();
            // Wait for threads to finish
            for t in 0 .. life.threadvec.len() {
                life.threadvec.pop().unwrap().join().unwrap();
            }
            life.gen_next();
            let dbuff = &life.dbuffs.0.lock().unwrap();

            let c = glgfx.draw_begin(args.viewport());
                render_polygons(&c.draw_state, &mut glgfx, &mut state, &mut polys, 1.0, Some(dbuff));
            glgfx.draw_end();

            state.tick().printfps(true); // Increment frame count
        },
        Event::Input( Input::Resize( ResizeArgs{window_size, draw_size} ), _ ) => {
            //println!("\x1b[1;31mEvent::Input::Resize::ResizeArgs {:?} {:?}", window_size, draw_size)
            state.W = window_size[0];
            state.H = window_size[1];
        },
        Event::Input( Input::Move( Motion::MouseCursor( [x, y]) ), _) => {
            //println!("\x1b[1;31mEvent::Input::Move::Motion::MouseCursor {:?} {:?} ", x as usize, y as usize);
            state.mx = (x - state.W/2.0) * 0.01;
            state.my = (y - state.H/2.0) * 0.01;
        },
        Event::Input( Input::Button( ButtonArgs{state:s, button:Button::Keyboard(k), scancode:_} ), _ ) => {
            //println!("Event::Input::Button == {:?} {:?} {:?}", s, b, c);
            match k {
                Key::Q => pwin.set_should_close(true),
                Key::S => { state.x += -0.05 * (state.mx).sin();  state.y += -0.05 * (state.mx).cos() },
                Key::A => { state.x +=  0.05 * (state.mx).sin();  state.y +=  0.05 * (state.mx).cos() },
                Key::D => { state.x +=  0.05 * (state.mx+1.570796).sin(); state.y +=  0.05 * (state.mx+1.570796).cos() },
                Key::F => { state.x +=  0.05 * (state.mx-1.570796).sin(); state.y +=  0.05 * (state.mx-1.570796).cos() },
                Key::V => { state.z -=  0.05 },
                Key::C => { state.z +=  0.05 },
                Key::Space => { ::util::sleep(500) },
                _ => ()
            }
        }, _ => ()
    } }  // match while
    Ok(0)
}

// Main ////////////////////////////////////////////////////////////////////////

pub fn main() {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    fun_piston().unwrap();
}

/* Notes ///////////////////////////////////////////////////////////////////////

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

     TRANSLATE
abcd   1..x   a b c ax+by+cz+d
efgh * .1.y = e f g ex+fy+gz+h
ijkl   ..1z   i j k ix+jy+kz+l
mnop   ...1   m n o mx+ny+oz+p

     SCALE
abcd  X...   aX bY cZ d
efgh *.Y.. = eX fY gZ h
ijkl  ..Z.   iX jY kZ l
mnop  ...1   mX nY oZ p

SCALE
X...   abcd   Xa Xb Xc Xd
.Y.. * efgh = Ye Yf Yg Yh
..Z.   ijkl   Zi Zj Zk Zl
...1   mnop   m  n  o  p

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
*///////////////////////////////////////////////////////////////////////////////