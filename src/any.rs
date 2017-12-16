use std::any::{Any as StdAny, TypeId};

pub trait Any: StdAny {
    fn type_id(&self) -> TypeId;

    fn type_name(&self) -> &str;
}

impl<T> Any for T
    where
        T: StdAny
{
    #[inline]
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    #[inline]
    fn type_name(&self) -> &str {
        use std::intrinsics::type_name;

        unsafe { type_name::<T>() }
    }
}

impl Any {
    #[inline]
    pub fn is<T: Any>(&self) -> bool {
        let t = TypeId::of::<T>();
        let boxed = self.type_id();

        t == boxed
    }

    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe {
                Some(&*(self as *const Any as *const T))
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe {
                Some(&mut *(self as *mut Any as *mut T))
            }
        } else {
            None
        }
    }
}
