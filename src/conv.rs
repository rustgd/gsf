use std::collections::VecDeque;

use {Any, Error, Result, Value, ValueTy, type_name_of};

pub type MultiVal<'a> = VecDeque<Value<'a>>;

pub trait FromMultiValue<'a>: Sized {
    fn ty() -> Vec<ValueTy>;

    fn from(v: MultiVal<'a>) -> Result<Self>;
}

pub trait FromValue<'a>: Sized {
    fn ty() -> ValueTy;

    fn from(v: Value<'a>) -> Result<Self>;
}

impl<'a> FromValue<'a> for u64 {
    fn ty() -> ValueTy {
        ValueTy::Int
    }

    fn from(v: Value<'a>) -> Result<Self> {
        match v.into_res()? {
            Value::Int(i) => Ok(i),
            other => Err(Error::WrongType {
                expected: Self::ty(),
                found: other.ty(),
            }),
        }
    }
}

impl<'a, T> FromValue<'a> for &'a T
where
    T: Any,
{
    fn ty() -> ValueTy {
        ValueTy::CustomRef
    }

    fn from(v: Value<'a>) -> Result<Self> {
        match v.into_res()? {
            Value::CustomRef(r) => r.downcast_ref().ok_or_else(|| Error::WrongAny {
                expected: type_name_of::<T>(),
                found: r.type_name(),
            }),
            other => Err(Error::WrongType {
                expected: Self::ty(),
                found: other.ty(),
            }),
        }
    }
}

impl<'a, T> FromValue<'a> for &'a mut T
    where
        T: Any,
{
    fn ty() -> ValueTy {
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
            },
            other => Err(Error::WrongType {
                expected: Self::ty(),
                found: other.ty(),
            }),
        }
    }
}

pub trait IntoValue: Sized {
    fn ty() -> ValueTy;

    fn into(self) -> Result<Value<'static>>;
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
            fn ty() -> Vec<ValueTy> {
                vec![ $( <$params as FromValue<'a>>::ty() ),* ]
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

def_from_multi!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
