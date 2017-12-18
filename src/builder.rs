use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::Arc;

use conv::{FromMultiValue, FromValue, IntoValue, MultiVal};
use {Any, Error, Function, Property, Ty, TyMap, TyMapMut, Value};

#[must_use]
#[derive(Default)]
pub struct Builder {
    pub map: TyMapMut,
}

impl Builder {
    pub fn add_ty<T: Any>(&mut self, builder: TyBuilder<T>) {
        self.map.insert(TypeId::of::<T>(), builder.ty);
    }

    pub fn with_ty<T: Any>(mut self, builder: TyBuilder<T>) -> Self {
        self.add_ty(builder);

        self
    }

    pub fn finish(self) -> TyMap {
        Arc::new(self.map)
    }
}

#[must_use]
pub struct TyBuilder<T> {
    pub marker: PhantomData<T>,
    pub ty: Ty,
}

impl<T> TyBuilder<T>
where
    T: Any + 'static,
{
    pub fn new(ident: &'static str) -> Self {
        TyBuilder {
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

    pub fn add_function<C, F, V>(&mut self, ident: &'static str, f: C)
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
            args: F::multi_ty(),
            ret: V::in_ty(),
        });
    }

    pub fn with_function<C, F, V>(mut self, ident: &'static str, f: C) -> Self
    where
        C: Fn(F) -> V + 'static,
        F: for<'a> FromMultiValue<'a>,
        V: IntoValue,
    {
        self.add_function(ident, f);

        self
    }

    pub fn add_method<C, F, V>(&mut self, ident: &'static str, f: C)
        where
            C: Fn(&T, F) -> V + 'static,
            F: for<'a> FromMultiValue<'a>,
            V: IntoValue,
    {
        let fptr = move |val: Vec<Value>| {
            let mut deque: MultiVal = val.into();
            let this = <&T as FromValue>::from(deque.pop_front().ok_or(Error::MissingSelfArg)?)?;

            let args = F::from(deque)?;
            let res = f(this, args);

            V::into(res)
        };
        let mut args = F::multi_ty();
        args.insert(0, <&T as FromValue>::out_ty());

        let fptr = move |val: Vec<Value>| fptr(val).into();
        self.ty.methods.push(Function {
            exec: Arc::new(fptr),
            ident: ident.into(),
            args,
            ret: V::in_ty(),
        });
    }

    pub fn with_method<C, F, V>(mut self, ident: &'static str, f: C) -> Self
    where
        C: Fn(&T, F) -> V + 'static,
        F: for<'a> FromMultiValue<'a>,
        V: IntoValue,
    {
        self.add_method(ident, f);

        self
    }

    pub fn add_method_mut<C, F, V>(&mut self, ident: &'static str, f: C)
        where
            C: Fn(&mut T, F) -> V + 'static,
            F: for<'a> FromMultiValue<'a>,
            V: IntoValue,
    {
        let fptr = move |val: Vec<Value>| {
            let mut deque: MultiVal = val.into();
            let this =
                <&mut T as FromValue>::from(deque.pop_front().ok_or(Error::MissingSelfArg)?)?;

            let args = F::from(deque)?;
            let res = f(this, args);

            V::into(res)
        };
        let fptr = move |val: Vec<Value>| fptr(val).into();
        let mut args = F::multi_ty();
        args.insert(0, <&mut T as FromValue>::out_ty());
        self.ty.methods.push(Function {
            exec: Arc::new(fptr),
            ident: ident.into(),
            args,
            ret: V::in_ty(),
        });
    }

    pub fn with_method_mut<C, F, V>(mut self, ident: &'static str, f: C) -> Self
    where
        C: Fn(&mut T, F) -> V + 'static,
        F: for<'a> FromMultiValue<'a>,
        V: IntoValue,
    {
        self.add_method_mut(ident, f);

        self
    }

    pub fn add_property<P>(&mut self, builder: PropertyBuilder<T, P>) -> Self {
        self.ty.properties.push(builder.property);

        self
    }

    pub fn with_property<P>(mut self, builder: PropertyBuilder<T, P>) -> Self {
        self.add_property(builder);

        self
    }
}

#[must_use]
pub struct PropertyBuilder<T, P> {
    property: Property,
    marker: PhantomData<(T, P)>,
}

impl<T, P> PropertyBuilder<T, P>
where
    T: Any,
    P: for<'a> FromValue<'a> + IntoValue,
{
    pub fn new(ident: &'static str) -> Self {
        PropertyBuilder {
            property: Property {
                ident: ident.into(),
                ty: P::out_ty(),
                get: None,
                set: None,
            },
            marker: PhantomData,
        }
    }

    pub fn add_getter<G>(&mut self, get: G)
    where
        G: Fn(&T) -> P + 'static,
    {
        let fptr = move |val: Vec<Value>| {
            let mut deque: MultiVal = val.into();
            let this = <&T as FromValue>::from(deque.pop_front().ok_or(Error::MissingSelfArg)?)?;

            P::into(get(this))
        };
        let fptr = move |val: Vec<Value>| fptr(val).into();

        self.property.get = Some(Arc::new(fptr));
    }

    pub fn with_getter<G>(mut self, get: G) -> Self
        where
            G: Fn(&T) -> P + 'static,
    {
        self.add_getter(get);

        self
    }

    pub fn add_setter<S>(&mut self, set: S)
        where
            S: Fn(&mut T, P) -> () + 'static,
    {
        let fptr = move |val: Vec<Value>| {
            let mut deque: MultiVal = val.into();
            let this =
                <&mut T as FromValue>::from(deque.pop_front().ok_or(Error::MissingSelfArg)?)?;

            let value = P::from(deque.pop_front().ok_or(Error::WrongArgsNumber {
                expected: 1,
                found: 0,
            })?)?;

            set(this, value);

            Ok(())
        };
        let fptr = move |val: Vec<Value>| fptr(val).map(|_| Value::Nil).into();

        self.property.set = Some(Arc::new(fptr));
    }

    pub fn with_setter<S>(mut self, set: S) -> Self
    where
        S: Fn(&mut T, P) -> () + 'static,
    {
        self.add_setter(set);

        self
    }
}
