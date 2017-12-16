#![feature(core_intrinsics)]

extern crate fnv;

use std::any::{Any as StdAny, TypeId};
use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

pub trait Any: StdAny {
    fn type_id(&self) -> TypeId;

    fn type_name(&self) -> &str;
}

impl<T> Any for T
where
    T: StdAny
{
    #[inline]
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    #[inline]
    fn type_name(&self) -> &str {
        use std::intrinsics::type_name;

        unsafe { type_name::<T>() }
    }
}

impl Any {
    #[inline]
    pub fn is<T: Any>(&self) -> bool {
        let t = TypeId::of::<T>();
        let boxed = self.type_id();

        t == boxed
    }

    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe {
                Some(&*(self as *const Any as *const T))
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe {
                Some(&mut *(self as *mut Any as *mut T))
            }
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Function {
    pub exec: FunPtr,
    pub ident: Str,
    //pub input: Value,
    //pub output: Value,
}

pub type FunPtr = fn(Vec<Value>) -> Value<'static>;

pub type Map<T> = fnv::FnvHashMap<Str, T>;

#[derive(Clone)]
pub struct Property {
    pub ident: Str,
    pub get: Option<FunPtr>,
    pub set: Option<FunPtr>,
}

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
    Tuple(Vec<Value<'a>>),
    Bool(bool),
    Int(u64),
    Float(f64),
    Custom(Box<Any>),
    CustomRef(&'a Any),
    CustomMut(&'a mut Any),
    Array(Vec<Value<'a>>),
    String(Str),
    Error,
}

impl<'a> fmt::Debug for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Tuple(ref v) => f.debug_tuple("Tuple").field(v).finish(),
            Value::Bool(ref b) => f.debug_tuple("Bool").field(b).finish(),
            Value::Int(ref i) => f.debug_tuple("Int").field(i).finish(),
            Value::Float(ref fl) => f.debug_tuple("Float").field(fl).finish(),
            Value::Custom(ref c) => f.debug_tuple("Custom").field(&c.type_id()).finish(),
            Value::CustomRef(_) => f.debug_tuple("CustomRef").finish(),
            Value::CustomMut(_) => f.debug_tuple("CustomMut").finish(),
            Value::Array(ref c) => f.debug_tuple("Array").field(c).finish(),
            Value::String(ref c) => f.debug_tuple("String").field(c).finish(),
            Value::Error => f.debug_tuple("Error").finish(),
        }
    }
}
