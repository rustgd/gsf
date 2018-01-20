extern crate gsf;
extern crate gsf_rlua;
extern crate rlua;

struct Foo(i32);

fn register(context: &rlua::Lua) -> rlua::Result<()> {
    let map = gsf::Builder::default()
        .with_ty(
            gsf::TyBuilder::<Foo>::new("Foo")
                .with_function("new", |(nr,): (i32,)| Box::new(Foo(nr as i32)))
                .with_function("sum_up", |(a, b): (i32, i32)| a + b)
                .with_method("foo_sq", |foo, ()| foo.0 * foo.0)
                .with_property(
                    gsf::PropertyBuilder::new("value")
                        .with_getter(|this: &Foo| this.0)
                        .with_setter(|this: &mut Foo, val: i32| this.0 = val),
                ),
        )
        .finish();
    gsf_rlua::register_with(context, &map)?;

    Ok(())
}

fn run() -> rlua::Result<()> {
    let context = rlua::Lua::new();
    register(&context)?;

    context.eval::<()>(r#"print(Foo.sum_up(1, 9))"#, Some("sum"))?;
    context.eval::<()>(r#"
Foo.sum_up(1) -- Does not work yet
print(Foo.sum_up(1, 1))
"#,
                       Some("error"))?;
    context.eval::<()>(r#"print(Foo.new(8):foo_sq())"#, Some("sq"))?;
    context.eval::<()>(r#"print(Foo.new(42):getValue())"#, Some("get"))?;
    context.eval::<()>(r#"print(Foo.new(42):setValue(43))"#, Some("set"))?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Failed to run: {}", e);
    }
}
