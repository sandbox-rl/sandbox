#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(private_interfaces)]

macro_rules! unreal_object {
    ($this:ty, $($super:ty,)? $pkg:literal, $name:literal) => {
        impl $crate::StaticClass for $this {
            const UNREAL_PACKAGE: &'static str = $pkg;
            const UNREAL_NAME: &'static str = $name;
        }

        impl $crate::UnrealObject for $this {}

        $(
            impl std::ops::Deref for $this {
                type Target = $super;

                fn deref(&self) -> &Self::Target {
                    &self._super
                }
            }

            impl std::ops::DerefMut for $this {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self._super
                }
            }
        )?
    }
}

mod core;
mod globals;

pub use core::*;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

const PROCESS_EVENT_INDEX: usize = 67;
const CALL_FUNCTION_INDEX: usize = 76;

type FPointer = *mut ();

#[repr(transparent)]
pub struct ueptr<T>(NonNull<T>);

impl<T> ueptr<T> {
    pub fn ptr_cast<U>(self) -> ueptr<U> {
        ueptr(self.0.cast())
    }

    pub fn ptr(self) -> *mut T {
        self.0.as_ptr()
    }
}

impl<T> PartialEq for ueptr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for ueptr<T> {}

impl<T> Clone for ueptr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ueptr<T> {}

impl<T> Deref for ueptr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for ueptr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

pub trait StaticClass {
    const UNREAL_PACKAGE: &'static str;
    const UNREAL_NAME: &'static str;

    fn StaticClass() -> ueptr<UClass> {
        UObject::FindClass(&format!(
            "Class {}::{}",
            Self::UNREAL_PACKAGE,
            Self::UNREAL_NAME
        ))
        .unwrap()
    }
}

pub trait UnrealObject: StaticClass {}
