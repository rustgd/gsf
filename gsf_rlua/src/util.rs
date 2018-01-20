use std::collections::VecDeque;

use gsf::{self, ValueTy};
use rlua::{self, Value};

use super::*;

pub fn to_lua_err(err: gsf::Error) -> rlua::Error {
    match err {
        gsf::Error::MissingSelfArg => rlua::Error::FromLuaConversionError {
            from: "missing argument",
            to: "self",
            message: None,
        },
        gsf::Error::WrongArgsNumber { expected, found } => rlua::Error::FromLuaConversionError {
            from: "missing argument",
            to: "expected arguments",
            message: Some(format!("Expected {} arguments, got {}", expected, found)),
        },
        gsf::Error::WrongType { expected, found } => rlua::Error::FromLuaConversionError {
            from: "lua value",
            to: "Rust value",
            message: Some(format!("Expected {:?}, got {:?}", expected, found)),
        },
        gsf::Error::WrongAny { .. } => unreachable!("This should never happen"),
    }
}

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
        ValueTy::Int8 => match val {
            Value::Integer(i) => f(gsf::Value::Int8(i as i8)),
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "integer 8-bit",
                message: Some(format!("Expected integer, got {:?}", other)),
            }),
        },
        ValueTy::Int16 => match val {
            Value::Integer(i) => f(gsf::Value::Int16(i as i16)),
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "integer 16-bit",
                message: Some(format!("Expected integer, got {:?}", other)),
            }),
        },
        ValueTy::Int32 => match val {
            Value::Integer(i) => f(gsf::Value::Int32(i as i32)),
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "integer 32-bit",
                message: Some(format!("Expected integer, got {:?}", other)),
            }),
        },
        ValueTy::Int64 => match val {
            Value::Integer(i) => f(gsf::Value::Int64(i as i64)),
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "integer 64-bit",
                message: Some(format!("Expected integer, got {:?}", other)),
            }),
        },
        ValueTy::Uint8 => match val {
            Value::Integer(i) => f(gsf::Value::Uint8(i as u8)),
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "unsigned integer 8-bit",
                message: Some(format!("Expected integer, got {:?}", other)),
            }),
        },
        ValueTy::Uint16 => match val {
            Value::Integer(i) => f(gsf::Value::Uint16(i as u16)),
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "unsigned integer 16-bit",
                message: Some(format!("Expected integer, got {:?}", other)),
            }),
        },
        ValueTy::Uint32 => match val {
            Value::Integer(i) => f(gsf::Value::Uint32(i as u32)),
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "unsigned integer 32-bit",
                message: Some(format!("Expected integer, got {:?}", other)),
            }),
        },
        ValueTy::Uint64 => match val {
            Value::Integer(i) => f(gsf::Value::Uint64(i as u64)),
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "unsigned integer 64-bit",
                message: Some(format!("Expected integer, got {:?}", other)),
            }),
        },
        ValueTy::Float32 => match val {
            Value::Number(nr) => f(gsf::Value::Float32(nr as f32)),
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "float 32-bit",
                message: Some(format!("Expected float, got {:?}", other)),
            }),
        },
        ValueTy::Float64 => match val {
            Value::Number(nr) => f(gsf::Value::Float64(nr as f64)),
            other => Err(rlua::Error::FromLuaConversionError {
                from: "value (TODO: use Value::type_name())",
                to: "float 64-bit",
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
