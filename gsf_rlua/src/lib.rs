extern crate gsf;
extern crate rlua;

mod util;

struct LuaUd(Box<gsf::Any>);

impl rlua::UserData for LuaUd {}

struct LuaFunc<'a> {
    func: &'a gsf::Function,
    tys: Vec<gsf::ValueTy>,
    map: gsf::TyMap,
}

impl<'a, 'lua> rlua::ToLua<'lua> for LuaFunc<'a> {
    fn to_lua(self, lua: &'lua rlua::Lua) -> rlua::Result<rlua::Value<'lua>> {
        let func = self.func.exec.clone();
        let map = self.map;
        let tys = self.tys;

        Ok(rlua::Value::Function(lua.create_function(
            move |lua, val: rlua::MultiValue| lua_func(&func, lua, &map, val, &tys),
        ).unwrap()))
    }
}

fn lua_func<'l>(
    fptr: &gsf::FunPtr,
    lua: &'l rlua::Lua,
    map: &gsf::TyMap,
    val: rlua::MultiValue<'l>,
    tys: &[gsf::ValueTy],
) -> rlua::Result<rlua::Value<'l>> {
    let res = lua_to_gsf_multi(val, tys, |args| gsf_to_lua(lua, fptr(args), &map));

    if let Err(ref e) = res {
        eprintln!("Returning error to lua: {}", e);
    }

    res
}

fn gsf_to_lua<'l>(
    lua: &'l rlua::Lua,
    val: gsf::Value,
    map: &gsf::TyMap,
) -> rlua::Result<rlua::Value<'l>> {
    let res = match val {
        gsf::Value::String(s) => rlua::Value::String(lua.create_string(&s)?),
        gsf::Value::Nil => rlua::Value::Nil,
        gsf::Value::Custom(b) => {
            let id = gsf::Any::type_id(b.as_ref());
            let ty = map.get(&id).ok_or(rlua::Error::ToLuaConversionError {
                from: "gsf Value",
                to: "User data",
                message: None,
            })?;

            let methods = &ty.methods;
            let props = &ty.properties;
            rlua::Value::UserData(lua.create_userdata_with_methods(
                LuaUd(b),
                to_methods(lua, &methods, &props, map),
            )?)
        }
        gsf::Value::Array(a) => {
            let table = lua.create_table()?;

            for (i, elem) in a.into_iter().enumerate() {
                table.set(i as i64 + 1, gsf_to_lua(lua, elem, map)?)?;
            }

            rlua::Value::Table(table)
        }
        gsf::Value::Bool(b) => rlua::Value::Boolean(b),
        gsf::Value::Int(x) => rlua::Value::Integer(x as i64),
        gsf::Value::Float(f) => rlua::Value::Number(f),
        gsf::Value::Error(e) => {
            eprintln!("Error: {:?}", e);
            rlua::Value::Error(rlua::Error::ToLuaConversionError {
                from: "",
                to: "",
                message: None,
            })
        }
        _ => unimplemented!(),
    };

    Ok(res)
}

fn lua_to_gsf_multi<F, R>(
    multi_val: rlua::MultiValue,
    tys: &[gsf::ValueTy],
    f: F,
) -> rlua::Result<R>
where
    F: FnOnce(Vec<gsf::Value>) -> rlua::Result<R>,
{
    util::convert_all(
        multi_val
            .into_inner()
            .into_iter()
            .zip(tys.iter().cloned())
            .collect(),
        f,
    )
}

fn to_methods<'l>(
    _: &'l rlua::Lua,
    funcs: &[gsf::Function],
    props: &[gsf::Property],
    map: &gsf::TyMap,
) -> rlua::UserDataMethods<'l, LuaUd> {
    let mut methods = rlua::UserDataMethods::default();

    for method in funcs {
        let fptr = method.exec.clone();
        let args = method.args.clone();
        let map = map.clone();
        methods.add_function(&method.ident, move |lua, val: rlua::MultiValue| {
            lua_func(&fptr, lua, &map, val, &args)
        });
    }

    for prop in props {
        let mut chars = prop.ident.chars();
        let s: String = chars
            .next()
            .map(move |c| c.to_uppercase().collect::<String>() + chars.as_str())
            .unwrap_or_default();

        if let Some(ref getter) = prop.get {
            let getter = getter.clone();
            let map = map.clone();
            methods.add_function(&format!("get{}", s), move |lua, val: rlua::MultiValue| {
                lua_func(&getter, lua, &map, val, &[gsf::ValueTy::CustomRef])
            });
        }

        if let Some(ref setter) = prop.set {
            let setter = setter.clone();
            let map = map.clone();
            let rt = prop.ty.clone();
            methods.add_function(&format!("set{}", s), move |lua, val: rlua::MultiValue| {
                lua_func(
                    &setter,
                    lua,
                    &map,
                    val,
                    &[gsf::ValueTy::CustomMut, rt.clone()],
                )
            });
        }
    }

    methods
}

fn register_ty(lua: &rlua::Lua, ty: &gsf::Ty, map: &gsf::TyMap) -> rlua::Result<()> {
    let table = lua.create_table()?;
    for f in &ty.functions {
        table.set(
            &f.ident as &str,
            LuaFunc {
                func: &f,
                map: map.clone(),
                tys: f.args.clone(),
            },
        )?;
    }

    let globals = lua.globals();
    globals.set(&ty.ident as &str, table)?;

    Ok(())
}

pub fn register_with(context: &rlua::Lua, map: &gsf::TyMap) -> rlua::Result<()> {
    for ty in map.values() {
        register_ty(context, ty, map)?;
    }

    Ok(())
}
