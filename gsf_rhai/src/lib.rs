extern crate gsf;
extern crate rhai;

use rhai::Engine;

fn register_ty(ty: gsf::Ty) {
    let mut engine = Engine::new();

    // TODO: "register" type
    // (currently a no-op)
    for fun in ty.functions {
        
    }
}
