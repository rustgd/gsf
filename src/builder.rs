use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::Arc;

use conv::{FromMultiValue, IntoValue};
use {Any, Function, Ty, TyMap, TyMapMut, Value};

#[derive(Default)]
pub struct Builder {
    pub map: TyMapMut,
}

impl Builder {
    pub fn build_ty<T: Any>(&mut self, ident: &'static str) -> TyBuilder<T> {
        TyBuilder {
            builder: self,
            marker: PhantomData,
            ty: Ty {
                functions: vec![],
                id: TypeId::of::<T>(),
                ident: ident.into(),
                methods: vec![],
                properties: vec![],
            },
        }
    }

    pub fn finish(self) -> TyMap {
        Arc::new(self.map)
    }
}

#[must_use]
pub struct TyBuilder<'b, T> {
    pub builder: &'b mut Builder,
    pub marker: PhantomData<T>,
    pub ty: Ty,
}

impl<'b, T: 'static> TyBuilder<'b, T> {
    pub fn add_function<C, F, V>(mut self, ident: &'static str, f: C) -> Self
    where
        C: Fn(F) -> V + 'static,
        F: for<'a> FromMultiValue<'a>,
        V: IntoValue,
    {
        let fptr = move |val: Vec<Value>| {
            let args = F::from(val.into())?;
            let res = f(args);

            V::into(res)
        };
        let fptr = move |val: Vec<Value>| fptr(val).into();
        self.ty.functions.push(Function {
            exec: Arc::new(fptr),
            ident: ident.into(),
            args: F::ty(),
            ret: V::ty(),
        });

        self
    }

    pub fn finish(self) {
        self.builder.map.insert(TypeId::of::<T>(), self.ty);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_function() {
        struct Foo;

        let mut builder = Builder::default();
        builder
            .build_ty::<Foo>("Foo")
            .add_function("do_it", |(a, b): (u64, u64)| println!("Summed up: {}", a + b));
    }
}