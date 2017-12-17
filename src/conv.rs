use std::collections::VecDeque;

use {type_name_of, Any, Error, Result, Str, Value, ValueTy};

pub type MultiVal<'a> = VecDeque<Value<'a>>;

pub trait FromMultiValue<'a>: Sized {
    fn multi_ty() -> Vec<ValueTy>;

    fn from(v: MultiVal<'a>) -> Result<Self>;
}

pub trait FromValue<'a>: Sized {
    fn out_ty() -> ValueTy;

    fn from(v: Value<'a>) -> Result<Self>;
}

macro_rules! def_from_val {
    ($fty:ty, $vty:ident) => {
def_from_val!($fty, $vty, $vty);
    };
    ($fty:ty, $vty:ident, $mat:ident) => {
def_from_val!($fty, $vty, $mat, Ok);
    };
    ($fty:ty, $vty:ident, $mat:ident, $to:expr) => {
impl<'a> FromValue<'a> for $fty {
    fn out_ty() -> ValueTy {
        ValueTy::$vty
    }

    fn from(v: Value<'a>) -> Result<Self> {
        match v.into_res()? {
            Value::$mat(mat_val) => ($to)(mat_val),
            other => Err(Error::WrongType {
                expected: Self::out_ty(),
                found: other.ty(),
            }),
        }
    }
}
    };
}

def_from_val!(u64, Int);
def_from_val!(f64, Float);
def_from_val!(bool, Bool);
def_from_val!(String, String, String, |s: Str| Ok(s.into_owned()));

impl<'a, T> FromValue<'a> for Option<T>
where
    T: FromValue<'a>
{
    fn out_ty() -> ValueTy {
        T::out_ty()
    }

    fn from(v: Value<'a>) -> Result<Self> {
        match v.into_res()? {
            Value::Nil => Ok(None),
            other => T::from(other).map(Some),
        }
    }
}

impl<'a, T> FromValue<'a> for &'a T
where
    T: Any,
{
    fn out_ty() -> ValueTy {
        ValueTy::CustomRef
    }

    fn from(v: Value<'a>) -> Result<Self> {
        match v.into_res()? {
            Value::CustomRef(r) => r.downcast_ref().ok_or_else(|| Error::WrongAny {
                expected: type_name_of::<T>(),
                found: r.type_name(),
            }),
            other => Err(Error::WrongType {
                expected: Self::out_ty(),
                found: other.ty(),
            }),
        }
    }
}

impl<'a, T> FromValue<'a> for &'a mut T
where
    T: Any,
{
    fn out_ty() -> ValueTy {
        ValueTy::CustomRef
    }

    fn from(v: Value<'a>) -> Result<Self> {
        match v.into_res()? {
            Value::CustomMut(r) => {
                let ty_name = (&*r).type_name();
                r.downcast_mut().ok_or(Error::WrongAny {
                    expected: type_name_of::<T>(),
                    found: ty_name.into(),
                })
            }
            other => Err(Error::WrongType {
                expected: Self::out_ty(),
                found: other.ty(),
            }),
        }
    }
}

pub trait IntoValue: Sized {
    fn in_ty() -> ValueTy;

    fn into(self) -> Result<Value<'static>>;
}

impl IntoValue for () {
    fn in_ty() -> ValueTy {
        ValueTy::Tuple(vec![])
    }

    fn into(self) -> Result<Value<'static>> {
        Ok(Value::Tuple(vec![]))
    }
}

impl IntoValue for u64 {
    fn in_ty() -> ValueTy {
        ValueTy::Int
    }

    fn into(self) -> Result<Value<'static>> {
        Ok(Value::Int(self))
    }
}

impl<T> IntoValue for Box<T>
where
    T: Any
{
    fn in_ty() -> ValueTy {
        ValueTy::Custom
    }

    fn into(self) -> Result<Value<'static>> {
        Ok(Value::Custom(self as Box<Any>))
    }
}

macro_rules! count_args {
    () => {0u16};
    ($head:ident $($tail:ident)*) => {1u16 + count_args!($($tail)*)};
}

macro_rules! def_from_multi {
    ($($params:ident),*) => {
        impl< 'a, $($params),* > FromMultiValue<'a> for ( $($params ,)* )
        where
            $( $params : FromValue<'a>),*
        {
            fn multi_ty() -> Vec<ValueTy> {
                vec![ $( <$params as FromValue<'a>>::out_ty() ),* ]
            }

            #[allow(unused_mut)]
            fn from(mut v: MultiVal<'a>) -> Result<Self> {
                let len = v.len() as u16;
                let expected = count_args!($($params)*);
                if len != expected {
                    return Err(Error::WrongArgsNumber {
                        expected,
                        found: len,
                    });
                }

                Ok(( $( <$params as FromValue<'a>>::from(v.pop_front().unwrap())? ,)* ))
            }
        }

        def_from_multi!(@ $($params),*);
    };
    (@) => {};
    (@ $head:ident $(,$tail:ident)*) => {
        def_from_multi!($($tail),*);
    };
}

#[cfg_attr(rustfmt, rustfmt_skip)]
def_from_multi!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
