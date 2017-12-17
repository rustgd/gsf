extern crate gsf;
extern crate gsf_rlua;
extern crate rlua;

struct Foo(i32);

fn register(context: &rlua::Lua) -> rlua::Result<()> {
    let mut builder = gsf::Builder::default();
    builder
        .build_ty::<Foo>("Foo")
        .add_function("sum_up", |(a, b): (u64, u64)| a + b)
        .finish();
    let map = builder.finish();
    gsf_rlua::register_with(context, &map)?;

    Ok(())
}

fn run() -> rlua::Result<()> {
    let context = rlua::Lua::new();
    register(&context)?;

    context.eval::<()>(r#"print("sum:", Foo.sum_up(1, 9))"#, None)?;

    Ok(())
}

fn main() {
    run().unwrap();
}
