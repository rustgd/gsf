use std::collections::VecDeque;

use gsf;
use rlua;

use super::*;

fn map<F, R>(val: rlua::Value, f: F) -> rlua::Result<R>
where
    F: FnOnce(gsf::Value) -> rlua::Result<R>,
{
    match val {
        // TODO: support signs
        rlua::Value::Integer(i) => f(gsf::Value::Int(i as u64)),
        rlua::Value::Number(nr) => f(gsf::Value::Float(nr)),
        rlua::Value::Boolean(b) => f(gsf::Value::Bool(b)),
        rlua::Value::String(s) => f(s.to_str()
            .map(ToOwned::to_owned)
            .map(Into::into)
            .map(gsf::Value::String)?),
        rlua::Value::Nil => f(gsf::Value::Tuple(Default::default())),
        rlua::Value::UserData(a) => {
            let ud = a.borrow::<LuaUd>()?;

            f(gsf::Value::CustomRef(ud.0.as_ref()))
        }
        _ => unimplemented!("does not support {:?}", val),
    }
}

pub fn convert_all<F, R>(v: VecDeque<rlua::Value>, f: F) -> rlua::Result<R>
where
    F: FnOnce(Vec<gsf::Value>) -> rlua::Result<R>,
{
    let len = v.len();
    convert_all_internal(v.into(), Vec::with_capacity(len), f)
}

fn convert_all_internal<F, R>(
    v: VecDeque<rlua::Value>,
    built: Vec<gsf::Value>,
    f: F,
) -> rlua::Result<R>
where
    F: FnOnce(Vec<gsf::Value>) -> rlua::Result<R>,
{
    match split(v) {
        None => f(built),
        Some((head, tail)) => map(head, move |val| {
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
