extern crate gsf;
extern crate gsf_rlua;
extern crate rlua;

use std::any::TypeId;
use std::sync::Arc;

struct Nobody(i32);

fn register(context: &rlua::Lua) -> rlua::Result<()> {
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
                    exec: |mut val| {
                        let val = match val.pop() {
                            Some(val) => val,
                            None => return gsf::Value::Error,
                        };

                        match val {
                            gsf::Value::CustomRef(a) => match gsf::Any::downcast_ref(a) {
                                Some(&Nobody(nr)) => gsf::Value::Int(nr as u64),
                                None => {
                                    eprintln!(
                                        "Type id: {:?}, type name: {:?}",
                                        a.type_id(),
                                        a.type_name()
                                    );

                                    gsf::Value::Error
                                }
                            },
                            _ => gsf::Value::Error,
                        }
                    },
                },
            ],
            properties: vec![
                gsf::Property {
                    ident: "square".into(),
                    get: Some(|mut val| {
                        let val = match val.pop() {
                            Some(val) => val,
                            None => return gsf::Value::Error,
                        };

                        match val {
                            gsf::Value::CustomRef(a) => match gsf::Any::downcast_ref(a) {
                                Some(&Nobody(nr)) => gsf::Value::Int((nr * nr) as u64),
                                None => {
                                    eprintln!(
                                        "Type id: {:?}, type name: {:?}",
                                        a.type_id(),
                                        a.type_name()
                                    );

                                    gsf::Value::Error
                                }
                            },
                            _ => gsf::Value::Error,
                        }
                    }),
                    set: None,
                },
            ],
        },
    );
    let map = Arc::new(map);
    gsf_rlua::register_with(context, &map)?;

    Ok(())
}

fn run() -> rlua::Result<()> {
    let context = rlua::Lua::new();
    register(&context)?;

    context.eval::<()>(r#"print("square:", Nobody.new():getSquare())"#, None)?;

    Ok(())
}

fn main() {
    run().unwrap();
}
