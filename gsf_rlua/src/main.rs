extern crate gsf;
extern crate gsf_rlua;
extern crate rlua;

struct Foo(i32);

fn register(context: &rlua::Lua) -> rlua::Result<()> {
    let mut builder = gsf::Builder::default();
    builder
        .build_ty::<Foo>("Foo")
        .add_function("new", |(nr,): (u64,)| Box::new(Foo(nr as i32)))
        .add_function("sum_up", |(a, b): (u64, u64)| a + b)
        .add_method("foo_sq", |foo, ()| (foo.0 * foo.0) as u64)
        .finish();
    let map = builder.finish();
    gsf_rlua::register_with(context, &map)?;

    Ok(())
}

fn run() -> rlua::Result<()> {
    let context = rlua::Lua::new();
    register(&context)?;

    context.eval::<()>(r#"print("sum:", Foo.sum_up(1, 9))"#, None)?;
    context.eval::<()>(r#"print("sq:", Foo.new(8):foo_sq())"#, None)?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Failed to run: {}", e);
    }
}
