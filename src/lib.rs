#![feature(core_intrinsics)]

extern crate fnv;

pub use any::{type_name_of, Any};
pub use builder::{Builder, PropertyBuilder, TyBuilder};

use std::any::TypeId;
use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

mod any;
mod builder;
mod conv;

#[derive(Clone, Debug)]
pub enum Error {
    MissingSelfArg,
    WrongArgsNumber {
        expected: u16,
        found: u16,
    },
    WrongType {
        expected: ValueTy,
        found: ValueTy,
    },
    WrongAny {
        expected: &'static str,
        found: &'static str,
    },
}

#[derive(Clone)]
pub struct Function {
    pub exec: FunPtr,
    pub ident: Str,
    pub args: Vec<ValueTy>,
    pub ret: ValueTy,
}

pub type FunPtr = Arc<Fn(Vec<Value>) -> Value<'static>>;

pub type Map<T> = fnv::FnvHashMap<Str, T>;

#[derive(Clone)]
pub struct Property {
    pub ident: Str,
    pub ty: ValueTy,
    pub get: Option<FunPtr>,
    pub set: Option<FunPtr>,
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub type Str = Cow<'static, str>;

#[derive(Clone)]
pub struct Ty {
    pub functions: Vec<Function>,
    pub id: TypeId,
    pub ident: Str,
    pub methods: Vec<Function>,
    pub properties: Vec<Property>,
}

pub type TyMap = Arc<TyMapMut>;
pub type TyMapMut = fnv::FnvHashMap<TypeId, Ty>;

pub enum Value<'a> {
    Nil,
    Void,
    Tuple(Vec<Value<'a>>),
    Bool(bool),
    Int(u64),
    Float(f64),
    Custom(Box<Any>),
    CustomRef(&'a Any),
    CustomMut(&'a mut Any),
    Array(Vec<Value<'a>>),
    String(Str),
    Error(Error),
}

impl<'a> Value<'a> {
    pub fn into_res(self) -> Result<Self> {
        self.into()
    }

    pub fn ty(&self) -> ValueTy {
        ValueTy::from(self)
    }
}

impl<'a> fmt::Debug for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Void => f.debug_tuple("Void").finish(),
            Value::Nil => f.debug_tuple("Nil").finish(),
            Value::Tuple(ref v) => f.debug_tuple("Tuple").field(v).finish(),
            Value::Bool(ref b) => f.debug_tuple("Bool").field(b).finish(),
            Value::Int(ref i) => f.debug_tuple("Int").field(i).finish(),
            Value::Float(ref fl) => f.debug_tuple("Float").field(fl).finish(),
            Value::Custom(ref c) => f.debug_tuple("Custom").field(&c.type_id()).finish(),
            Value::CustomRef(_) => f.debug_tuple("CustomRef").finish(),
            Value::CustomMut(_) => f.debug_tuple("CustomMut").finish(),
            Value::Array(ref c) => f.debug_tuple("Array").field(c).finish(),
            Value::String(ref c) => f.debug_tuple("String").field(c).finish(),
            Value::Error(ref e) => f.debug_tuple("Error").field(e).finish(),
        }
    }
}

impl<'a> From<Result<Value<'a>>> for Value<'a> {
    fn from(res: Result<Value<'a>>) -> Self {
        res.unwrap_or_else(|e| Value::Error(e))
    }
}

impl<'a> Into<Result<Value<'a>>> for Value<'a> {
    fn into(self) -> Result<Value<'a>> {
        match self {
            Value::Error(e) => Err(e),
            val => Ok(val),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ValueTy {
    Unknown,
    Void,
    Tuple(Vec<ValueTy>), // TODO: remove
    Bool,
    Int,
    Float,
    Custom,
    CustomRef,
    CustomMut,
    Option(Box<ValueTy>),
    Array(Box<ValueTy>),
    String,
}

impl<'a, 'b> From<&'a Value<'b>> for ValueTy {
    fn from(val: &Value) -> Self {
        match *val {
            Value::Nil => ValueTy::Unknown,
            Value::Void => ValueTy::Void,
            Value::Tuple(ref v) => ValueTy::Tuple(v.iter().map(From::from).collect()),
            Value::Bool(_) => ValueTy::Bool,
            Value::Int(_) => ValueTy::Int,
            Value::Float(_) => ValueTy::Float,
            Value::Custom(_) => ValueTy::Custom,
            Value::CustomRef(_) => ValueTy::CustomRef,
            Value::CustomMut(_) => ValueTy::CustomMut,
            Value::Array(ref a) => a.iter().next().map(From::from).unwrap_or(ValueTy::Unknown),
            Value::String(_) => ValueTy::String,
            Value::Error(_) => ValueTy::Unknown,
        }
    }
}
