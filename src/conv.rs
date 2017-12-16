use std::collections::VecDeque;

use {Value, ValueTy};

pub type MultiVal<'a> = VecDeque<Value<'a>>;

pub trait FromMultiValue {
    fn ty() -> Vec<ValueTy>;

    fn from(v: MultiVal) -> Self;
}

pub trait FromValue {
    fn ty() -> ValueTy;

    fn from(v: Value) -> Self;
}

pub trait IntoValue {
    fn ty() -> ValueTy;

    fn into(self) -> Value<'static>;
}

macro_rules! def_from_multi {
    ($($params:ident),*) => {
        impl< $($params),* > FromMultiValue for ( $($params ,)* )
        where
            $($params : FromValue),*
        {
            fn ty() -> Vec<ValueTy> {
                vec![ $( <$params as FromValue>::ty() ),* ]
            }

            fn from(mut _v: MultiVal) -> Self {
                ( $( <$params as FromValue>::from(_v.pop_front().unwrap()) ,)* )
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
