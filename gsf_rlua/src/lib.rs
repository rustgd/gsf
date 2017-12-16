#![feature(core_instrinsics)]

extern crate gsf;
extern crate rlua;

use std::any::TypeId;
use std::sync::Arc;

struct LuaUd(Box<gsf::Any>);

impl rlua::UserData for LuaUd {}

struct LuaFunc<'a>(&'a gsf::Function, gsf::TyMap);

impl<'a, 'lua> rlua::ToLua<'lua> for LuaFunc<'a> {
    fn to_lua(self, lua: &'lua rlua::Lua) -> rlua::Result<rlua::Value<'lua>> {
        let func = self.0.exec;
        let map = self.1;

        Ok(rlua::Value::Function(lua.create_function(
            move |lua, val: rlua::MultiValue| {
                println!("val: {:?}", val);

                let val = func(gsf::Value::Tuple(Default::default()));
                gsf_to_lua(lua, val, &map)
            },
        ).unwrap()))
    }
}

fn gsf_to_lua<'l>(
    lua: &'l rlua::Lua,
    val: gsf::Value,
    map: &gsf::TyMap,
) -> rlua::Result<rlua::Value<'l>> {
    let res = match val {
        gsf::Value::String(s) => rlua::Value::String(lua.create_string(&s)?),
        gsf::Value::Tuple(ref v) if v.is_empty() => rlua::Value::Nil,
        gsf::Value::Custom(b) => {
            let id = gsf::Any::type_id(b.as_ref());
            let ty = map.get(&id).ok_or(rlua::Error::ToLuaConversionError {
                from: "gsf Value",
                to: "User data",
                message: None,
            })?;

            let methods = &ty.methods;
            rlua::Value::UserData(lua.create_userdata_with_methods(
                LuaUd(b),
                to_methods(lua, &methods, map),
            )?)
        }
        gsf::Value::Int(x) => rlua::Value::Integer(x as i64),
        gsf::Value::Error => rlua::Value::Error(rlua::Error::ToLuaConversionError {
            from: "",
            to: "",
            message: None,
        }),
        _ => unimplemented!(),
    };

    Ok(res)
}

fn lua_to_gsf<F, R>(val: rlua::MultiValue, f: F) -> R
where
    F: FnOnce(rlua::Result<gsf::Value>) -> R,
{
    let val = val.into_iter().next().unwrap();

    match val {
        // TODO: support signs
        rlua::Value::Integer(i) => f(Ok(gsf::Value::Int(i as u64))),
        rlua::Value::String(s) => f(s.to_str()
            .map(ToOwned::to_owned)
            .map(Into::into)
            .map(gsf::Value::String)),
        rlua::Value::Nil => f(Ok(gsf::Value::Tuple(Default::default()))),
        rlua::Value::UserData(a) => {
            let ud = a.borrow::<LuaUd>();
            match ud {
                Ok(ud) => f(Ok(gsf::Value::CustomRef(ud.0.as_ref()))),
                Err(e) => f(Err(e)),
            }
        }
        _ => unimplemented!("does not support {:?}", val),
    }
}

fn to_methods<'l>(
    _: &'l rlua::Lua,
    funcs: &[gsf::Function],
    map: &gsf::TyMap,
) -> rlua::UserDataMethods<'l, LuaUd> {
    let mut methods = rlua::UserDataMethods::default();

    for method in funcs {
        let fptr = method.exec;
        let map = map.clone();
        methods.add_function(&method.ident, move |lua, val: rlua::MultiValue| {
            println!("val: {:?}", val);

            lua_to_gsf(val, |args| gsf_to_lua(lua, fptr(args?), &map))
        });
    }

    methods
}

fn register_ty(lua: &rlua::Lua, ty: &gsf::Ty, map: &gsf::TyMap) -> rlua::Result<()> {
    let table = lua.create_table()?;
    for f in &ty.functions {
        table.set(&f.ident as &str, LuaFunc(&f, map.clone()))?;
    }

    let globals = lua.globals();
    globals.set(&ty.ident as &str, table)?;

    Ok(())
}

struct Nobody(i32);

pub fn run() -> rlua::Result<()> {
    let context = rlua::Lua::new();

    let mut map = gsf::TyMapMut::default();
    map.insert(
        TypeId::of::<Nobody>(),
        gsf::Ty {
            functions: vec![
                gsf::Function {
                    exec: |_| gsf::Value::Custom(Box::new(Nobody(4))),
                    ident: "new".into(),
                },
            ],
            id: TypeId::of::<Nobody>(),
            ident: "Nobody".into(),
            methods: vec![
                gsf::Function {
                    ident: "saySomething".into(),
                    exec: |val| {
                        println!("val: {:?}", val);

                        match val {
                            gsf::Value::CustomRef(a) => match gsf::Any::downcast_ref(a) {
                                Some(&Nobody(nr)) => gsf::Value::Int(nr as u64),
                                None => {
                                    eprintln!("Type id: {:?}, type name: {:?}",
                                              a.type_id(),
                                              a.type_name());

                                    gsf::Value::Error
                                },
                            }
                            _ => gsf::Value::Error,
                        }
                    },
                },
            ],
            properties: vec![],
        },
    );
    let map = Arc::new(map);

    for ty in map.values() {
        println!("Registering type with type id {:?}", ty.id);
        register_ty(&context, ty, &map)?;
    }

    context.eval::<()>(r#"print(Nobody.new():saySomething())"#, None)?;

    Ok(())
}
