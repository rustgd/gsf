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
        ValueTy::Int => "::std::os::raw::c_int",
        _ => unimplemented!(),
    }
}

fn write_ty_decl<W>(ty: &gsf::Ty, mut to: W) -> io::Result<()> {
    write!(
        to,
        "\
#[repr(C)]
pub struct {ident} {{
    _inner: Box<::gsf::Any>,
}}",
        ident = ty.ident,
    )
}

fn write_fun<W>(ty_ident: &str, f: gsf::Function, mut to: W) -> io::Result<()>
where
    W: Write,
{
    let args: String = f.args
        .iter()
        .map(to_rust_ty)
        .enumerate()
        .map(|(i, x)| format!("_arg{}: {},", i, x))
        .collect();
    let pass_args: String = (0..f.args.len()).map(|x| format!("_arg{},", x)).collect();

    let f_ident = format!("{}_{}", ty_ident, f.ident);

    write!(
        to,
        "\
#[no_mangle]\
pub extern \"C\" fn {ident}({args})
-> {rt} {{
\t{body}
}}",
        rt = to_rust_ty(&f.ret),
        ident = f_ident,
        args = args,
        body = format!("(__GSF_CONTEXT.{})({})", f_ident, pass_args),
    )
}

mod tests {
    use super::*;

    #[test]
    fn conv_func() {
        let mut v: Vec<u8> = Vec::new();

        write_fun(
            "Foo",
            gsf::Function {
                exec: Arc::new(|_| gsf::Value::Nil),
                ident: "foo".into(),
                args: vec![ValueTy::Int, ValueTy::Int],
                ret: ValueTy::Void,
            },
            &mut v,
        ).unwrap();

        println!("{}", String::from_utf8(v).unwrap());

        assert!(false);
    }
}
