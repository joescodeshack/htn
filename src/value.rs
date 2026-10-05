use std::cmp::Ordering;

/// A blackboard value.
///
/// Values of different types are never equal or ordered: `Int(1) != Float(1.0)`.
/// Conditions that compare different types fail with
/// [`PlanError::TypeMismatch`](crate::PlanError::TypeMismatch).
#[derive(Clone, Debug, PartialEq)]
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

// Only values of the same type are ordered, matching the derived `PartialEq`.
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a.partial_cmp(b),
            (Self::Int(a), Self::Int(b)) => a.partial_cmp(b),
            (Self::Float(a), Self::Float(b)) => a.partial_cmp(b),
            (Self::String(a), Self::String(b)) => a.partial_cmp(b),
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
