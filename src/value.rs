use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub enum Value {
    Int(i32),
    Float(f32),
    Bool(bool),
    String(String),
    Vector3(f32, f32, f32),
    Vector2(f32, f32),
}

macro_rules! impl_from {
    ($variant:ident, $ty:ty) => {
        impl From<$ty> for Value {
            fn from(v: $ty) -> Self {
                Value::$variant(v)
            }
        }
    };

    ($variant:ident, $ty:ty, |$v:ident| $body:expr) => {
        impl From<$ty> for Value {
            fn from($v: $ty) -> Self {
                Value::$variant($body)
            }
        }
    };
}

impl_from!(Int, i32);
impl_from!(Float, f32);
impl_from!(Bool, bool);
impl_from!(String, String);
impl_from!(String, &str, |v| v.to_string());

// Defined through `partial_cmp` so `==` always agrees with `<=` and `>=`
// (e.g. `Int(1) == Float(1.0)`).
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a.partial_cmp(b),
            (Self::Int(a), Self::Int(b)) => a.partial_cmp(b),
            (Self::Float(a), Self::Float(b)) => a.partial_cmp(b),
            (Self::String(a), Self::String(b)) => a.partial_cmp(b),
            (Self::Int(a), Self::Float(b)) => (*a as f32).partial_cmp(b),
            (Self::Float(a), Self::Int(b)) => a.partial_cmp(&(*b as f32)),
            // Vectors have no ordering; they are only ever equal or unordered.
            (Self::Vector2(a1, a2), Self::Vector2(b1, b2)) => {
                ((a1, a2) == (b1, b2)).then_some(Ordering::Equal)
            }
            (Self::Vector3(a1, a2, a3), Self::Vector3(b1, b2, b3)) => {
                ((a1, a2, a3) == (b1, b2, b3)).then_some(Ordering::Equal)
            }
            _ => None,
        }
    }
}
