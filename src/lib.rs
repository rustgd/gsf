#![feature(core_intrinsics)]

extern crate fnv;

pub use any::{type_name_of, Any};
pub use builder::{Builder, PropertyBuilder, TyBuilder};
pub use conv::{FromValue, FromMultiValue, IntoValue, MultiVal};

use std::any::TypeId;
use std::borrow::Cow;
use std::error;
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

impl error::Error for Error {
    fn description(&self) -> &str {
        "gsf Error"
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    Float32(f32),
    Float64(f64),
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
            Value::Int8(ref i) => f.debug_tuple("Int8").field(i).finish(),
            Value::Int16(ref i) => f.debug_tuple("Int16").field(i).finish(),
            Value::Int32(ref i) => f.debug_tuple("Int32").field(i).finish(),
            Value::Int64(ref i) => f.debug_tuple("Int64").field(i).finish(),
            Value::Uint8(ref i) => f.debug_tuple("Uint8").field(i).finish(),
            Value::Uint16(ref i) => f.debug_tuple("Uint16").field(i).finish(),
            Value::Uint32(ref i) => f.debug_tuple("Uint32").field(i).finish(),
            Value::Uint64(ref i) => f.debug_tuple("Uint64").field(i).finish(),
            Value::Float32(ref fl) => f.debug_tuple("Float32").field(fl).finish(),
            Value::Float64(ref fl) => f.debug_tuple("Float64").field(fl).finish(),
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
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float32,
    Float64,
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
            Value::Int8(_) => ValueTy::Int8,
            Value::Int16(_) => ValueTy::Int16,
            Value::Int32(_) => ValueTy::Int32,
            Value::Int64(_) => ValueTy::Int64,
            Value::Uint8(_) => ValueTy::Uint8,
            Value::Uint16(_) => ValueTy::Uint16,
            Value::Uint32(_) => ValueTy::Uint32,
            Value::Uint64(_) => ValueTy::Uint64,
            Value::Float32(_) => ValueTy::Float32,
            Value::Float64(_) => ValueTy::Float64,
            Value::Custom(_) => ValueTy::Custom,
            Value::CustomRef(_) => ValueTy::CustomRef,
            Value::CustomMut(_) => ValueTy::CustomMut,
            Value::Array(ref a) => a.iter().next().map(From::from).unwrap_or(ValueTy::Unknown),
            Value::String(_) => ValueTy::String,
            Value::Error(_) => ValueTy::Unknown,
        }
    }
}
