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
                from: "TODO: use Value::type_name()",
                to: "boolean",
                message: Some(format!("Expected boolean, got {}", other.to_string())),
            })
        },
        ValueTy::Int => {},
        ValueTy::Float => {},
        ValueTy::Custom => {},
        ValueTy::CustomRef => {},
        ValueTy::CustomMut => {},
        ValueTy::String => {},
        ValueTy::Option(o) => match val {
            Value::Nil => f(gsf::Value::Nil),
            other => map(other, o, f),
        },
        ValueTy::Array(_) => unimplemented!(),
        ValueTy::Tuple(_) => unimplemented!(),
        ValueTy::Unknown => unimplemented!(),
    }
    /*match val {
        // TODO: support signs
        rlua::Value::Integer(i) => f(gsf::Value::Int(i as u64)),
        rlua::Value::Number(nr) => f(gsf::Value::Float(nr)),
        rlua::Value::Boolean(b) => f(gsf::Value::Bool(b)),
        rlua::Value::String(s) => f(s.to_str()
            .map(ToOwned::to_owned)
            .map(Into::into)
            .map(gsf::Value::String)?),
        rlua::Value::Nil => f(gsf::Value::Nil),
        rlua::Value::UserData(a) => match ty {
            gsf::ValueTy::CustomRef => {
                let ud = a.borrow::<LuaUd>()?;

                f(gsf::Value::CustomRef(ud.0.as_ref()))
            }
            gsf::ValueTy::CustomMut => {
                let mut ud = a.borrow_mut::<LuaUd>()?;

                f(gsf::Value::CustomMut(ud.0.as_mut()))
            }
            _ => Err(rlua::Error::FromLuaConversionError {
                from: "UserData",
                to: "Non UserData",
                message: Some("Script passed user data, but no user data was expected".to_owned()),
            }),
        },
        _ => unimplemented!("does not support {:?}", val),
    }*/
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
