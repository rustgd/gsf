extern crate gsf;

use std::io::{self, Write};
use std::sync::Arc;

use gsf::ValueTy;

fn to_c_ty(ty: &gsf::ValueTy) -> &'static str {
    match *ty {
        ValueTy::Void => "void",
        _ => unimplemented!(),
    }
}

fn to_rust_ty(ty: &gsf::ValueTy) -> &'static str {
    match *ty {
        ValueTy::Void => "()",
        ValueTy::Uint64 => "u64",
        _ => unimplemented!(),
    }
}

fn write_ty_decl<W>(ty: &gsf::Ty, mut to: W) -> io::Result<()>
where
    W: Write,
{
    write!(
        to,
        "\
#[repr(C)]
pub struct {ident} {{
    _inner: *mut (),
}}",
        ident = ty.ident,
    )
}

struct FunMeta {
    args: String,
    pass_args: String,
    ty_args: String,
    f_ident: String,
    ret: &'static str,
}

fn convert_fun(ty_ident: &str, f: gsf::Function) -> FunMeta {
    let args = f.args
        .iter()
        .map(to_rust_ty)
        .enumerate()
        .map(|(i, x)| format!("_arg{}: {}, ", i, x))
        .collect();

    let ty_args = f.args
        .iter()
        .map(to_rust_ty)
        .map(|x| format!("{}, ", x))
        .collect();

    let pass_args: String = (0..f.args.len()).map(|x| format!("_arg{},", x)).collect();
    let f_ident = format!("{}_{}", ty_ident, f.ident);
    let ret = to_rust_ty(&f.ret);

    FunMeta {
        args,
        pass_args,
        ty_args,
        f_ident,
        ret,
    }
}

fn write_fun<W>(meta: &FunMeta, mut to: W) -> io::Result<()>
where
    W: Write,
{
    write!(
        to,
        "\
#[no_mangle]
pub unsafe extern \"C\" fn {ident}({args})
-> {rt} {{
\t{body}
}}",
        rt = meta.ret,
        ident = meta.f_ident,
        args = meta.args,
        body = format!("((&*__GSF_CONTEXT).{} )({})", meta.f_ident, meta.pass_args),
    )
}

fn write_context<W>(meta: &[&FunMeta], mut to: W) -> io::Result<()>
where
    W: Write,
{
    let body: String = meta.iter()
        .map(|meta: &&FunMeta| {
            format!(
                "    {ident}: ::std::sync::Arc<Fn({args})>,\n",
                ident = meta.f_ident,
                args = meta.ty_args
            )
        })
        .collect();
    write!(
        to,
        "pub struct __GSF_CONTEXT_TY {{\n{body}}}
pub static mut __GSF_CONTEXT: *const __GSF_CONTEXT_TY = 0 as *const __GSF_CONTEXT_TY;
",
        body = body
    )
}

fn write_header<W>(mut to: W) -> io::Result<()>
where
    W: Write,
{
    write!(to, "#![allow(bad_style)]\n\n")
}

mod tests {
    use super::*;

    #[test]
    fn conv_func() {
        let mut v: Vec<u8> = Vec::new();

        let meta = convert_fun(
            "Foo",
            gsf::Function {
                exec: Arc::new(|_| gsf::Value::Nil),
                ident: "foo".into(),
                args: vec![ValueTy::Uint64, ValueTy::Uint64],
                ret: ValueTy::Void,
            },
        );

        write_header(&mut v).unwrap();
        write_context(&[&meta], &mut v).unwrap();
        write_fun(&meta, &mut v).unwrap();

        println!("{}", String::from_utf8(v).unwrap());

        assert!(false);
    }
}
