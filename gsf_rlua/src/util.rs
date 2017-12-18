use std::collections::VecDeque;

use gsf::{self, ValueTy};
use rlua::{self, Value};

use super::*;

fn map<F, R>(val: rlua::Value, ty: ValueTy, f: F) -> rlua::Result<R>
where
    F: FnOnce(gsf::Value) -> rlua::Result<R>,
{
    match ty {
        ValueTy::Bool => match val {
            Value::Boolean(b) => f(gsf::Value::Bool(b)),
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "boolean",
                message: Some(format!("Expected boolean, got {:?}", other)),
            }),
        },
        ValueTy::Int => match val {
            Value::Integer(i) => f(gsf::Value::Int(i as u64)),
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "integer",
                message: Some(format!("Expected integer, got {:?}", other)),
            }),
        },
        ValueTy::Float => match val {
            Value::Number(nr) => f(gsf::Value::Float(nr)),
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "float",
                message: Some(format!("Expected float, got {:?}", other)),
            }),
        },
        ValueTy::CustomRef => match val {
            Value::UserData(ud) => {
                let ud = ud.borrow::<LuaUd>()?;
                f(gsf::Value::CustomRef(ud.0.as_ref()))
            }
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "custom ref",
                message: Some(format!("Expected custom ref, got {:?}", other)),
            }),
        },
        ValueTy::CustomMut => match val {
            Value::UserData(ud) => {
                let mut ud = ud.borrow_mut::<LuaUd>()?;
                f(gsf::Value::CustomMut(ud.0.as_mut()))
            }
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "custom mut",
                message: Some(format!("Expected custom mut, got {:?}", other)),
            }),
        },
        ValueTy::String => match val {
            Value::String(s) => f(gsf::Value::String(s.to_str()?.to_owned().into())),
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "string",
                message: Some(format!("Expected string, got {:?}", other)),
            }),
        },
        ValueTy::Option(o) => match val {
            Value::Nil => f(gsf::Value::Nil),
            other => map(other, *o, f),
        },
        ValueTy::Void => unimplemented!(),
        ValueTy::Custom => unimplemented!(),
        ValueTy::Array(_) => unimplemented!(),
        ValueTy::Tuple(_) => unimplemented!(),
        ValueTy::Unknown => unimplemented!(),
    }
}

pub fn convert_all<F, R>(v: VecDeque<(rlua::Value, gsf::ValueTy)>, f: F) -> rlua::Result<R>
where
    F: FnOnce(Vec<gsf::Value>) -> rlua::Result<R>,
{
    let len = v.len();
    convert_all_internal(v, Vec::with_capacity(len), f)
}

fn convert_all_internal<F, R>(
    v: VecDeque<(rlua::Value, gsf::ValueTy)>,
    built: Vec<gsf::Value>,
    f: F,
) -> rlua::Result<R>
where
    F: FnOnce(Vec<gsf::Value>) -> rlua::Result<R>,
{
    match split(v) {
        None => f(built),
        Some((head, tail)) => map(head.0, head.1, move |val| {
            convert_all_internal(tail, combine(built, val), f)
        }),
    }
}

fn combine<T>(mut v: Vec<T>, elem: T) -> Vec<T> {
    v.push(elem);

    v
}

fn split<T>(mut v: VecDeque<T>) -> Option<(T, VecDeque<T>)> {
    v.pop_front().map(|elem| (elem, v))
}
