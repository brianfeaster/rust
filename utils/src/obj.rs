use std::ops::{BitAnd};
use ::std::{fmt};

#[derive(Clone)]
pub enum Obj {
    Null,
    Num(i64),
    Str(String),
    Cons(Box<(Obj,Obj)>),
    Vec(Vec<Obj>)
}

impl From<i64> for Obj { fn from(n: i64) -> Self { Obj::Num(n as i64) } }
impl From<i32> for Obj { fn from(n: i32) -> Self { Obj::Num(n as i64) } }
impl From<&str> for Obj { fn from(s: &str) -> Self { Obj::Str(s.to_string()) } }
impl From<()> for Obj { fn from(_:()) -> Self { Obj::Null } }
impl From<(Obj,Obj)> for Obj { fn from(p:(Obj,Obj)) -> Self { Obj::Cons(Box::new(p)) } }
impl From<&[Obj]> for Obj { fn from(sl:&[Obj]) -> Self { Obj::Vec(sl.to_vec()) } }


fn fmt_list (
        this:  &Obj,
        first: bool,
        f:     &mut fmt::Formatter
) -> fmt::Result {
    match this {
        Obj::Cons(bpair) => {
            if !first { write!(f, " ").unwrap(); }
            write!(f, "{}", bpair.0).unwrap(); // Out of list context
            fmt_list(&bpair.1, false, f) // Continue list context
        },
        Obj::Null =>
            write!(f,""),
        _ =>
            write!(f, " . {}", this)
    }
}

impl fmt::Display for Obj {
    fn fmt(
        &self,
        f: &mut fmt::Formatter
    ) -> fmt::Result {
        match self {
            Obj::Null =>
                write!(f, "()"),
            Obj::Num(n) =>
                write!(f, "{}", n),
            Obj::Str(s) =>
                write!(f, "\"{}\"", s),
            Obj::Cons(_) => {
                write!(f, "(").unwrap();
                fmt_list(self, true, f).unwrap();
                write!(f, ")")
            },
            Obj::Vec(v) => {
                write!(f, "#(").unwrap();
                let mut first=true;
                for o in v {
                    if !first { write!(f," ").unwrap(); }
                    write!(f, "{}", o).unwrap();
                    first = false;
                }
                write!(f, ")")
            }
        }
    }
}

pub fn null ()             -> Obj { Obj::Null }
pub fn num  (n:i64)        -> Obj { Obj::from(n) }
pub fn str  (s:&str)       -> Obj { Obj::from(s) }
pub fn vec  (sl:&[Obj])    -> Obj { Obj::from(sl) }
pub fn cons (a:Obj, b:Obj) -> Obj { Obj::from((a, b)) }
pub fn list (lst:&[Obj]) -> Obj {
    if 0 == lst.len() {
        null()
    } else {
        cons(lst[0].clone(), list(&lst[1..]))
    }
}

// There's no non-assigning right associative
// op to overload so this is clunky and requires
// parens for the rhs.
impl BitAnd<Obj> for Obj {
    type Output = Obj;
    fn bitand(self, rhs: Obj) -> Obj { cons(self, rhs) }
}

pub fn test () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());

    // Pair of two objects  (1 . "hi")
    println!("pair {}", Obj::Cons(Box::new((Obj::Num(1), Obj::Str("hi".to_string())))) );
    println!("pair {}", Obj::from((Obj::from(1), Obj::from("hi")) ));
    println!("pair {}", cons(num(1), str("hi")) );
    println!("pair {}", num(1) & str("hi") );

    // List of 2 objects  (1 "hi")
    println!("list {}", cons(num(1), cons(str("hi"), null())) );
    println!("list {}", num(1) & (str("hi") & null()) );
    println!("list {}", list( &[ num(1), num(2) ]) );

    // List of 3 sexprs  ((42) (64 . "a") (69 () . 0))
    println!("sexpr {}", 
        list(&[num(42)])
        &
        ( num(64) & str("a")
          & ((num(69) & (null() & num(0)))
             & null())));

    println!("sexpr {}", list( &[num(0), vec( &[num(1), num(2), str("vec"), null()][..] )] ));
}