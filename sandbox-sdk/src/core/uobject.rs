use std::collections::HashMap;
use std::sync::OnceLock;
use std::{ffi::c_void, ptr::NonNull};
use std::{iter, ptr};

use bitflags::bitflags;

use crate::{
    ueptr, FFrame, FName, FPointer, TArray, UClass, UFunction, UnrealObject, CALL_FUNCTION_INDEX,
    PROCESS_EVENT_INDEX,
};

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct EObjectFlags: u64 {
        const NoFlags = 0x0000_0000;
        const Public = 0x0000_0001;
        const Standalone = 0x0000_0002;
        const MarkAsNative = 0x0000_0004;
        const Transactional = 0x0000_0008;
        const ClassDefaultObject = 0x0000_0010;
        const ArchetypeObject = 0x0000_0020;
        const Transient = 0x0000_0040;
        const MarkAsRootSet = 0x0000_0080;
        const TagGarbageTemp = 0x0000_0100;
        const NeedInitialization = 0x0000_0200;
        const NeedLoad = 0x0000_0400;
        const KeepForCooker = 0x0000_0800;
        const NeedPostLoad = 0x0000_1000;
        const NeedPostLoadSubobjects = 0x0000_2000;
        const NewerVersionExists = 0x0000_4000;
        const BeginDestroyed = 0x0000_8000;
        const FinishDestroyed = 0x0001_0000;
        const BeingRegenerated = 0x0002_0000;
        const DefaultSubObject = 0x0004_0000;
        const WasLoaded = 0x0008_0000;
        const TextExportTransient = 0x0010_0000;
        const LoadCompleted = 0x0020_0000;
        const InheritableComponentTemplate = 0x0040_0000;
        const DuplicateTransient = 0x0080_0000;
        const StrongRefOnFrame = 0x0100_0000;
        const NonPIEDuplicateTransient = 0x0200_0000;
        const Dynamic = 0x0400_0000;
        const WillBeLoaded = 0x0800_0000;
    }
}

#[repr(C)]
struct UObjectVtbl {
    _pad_0: [*mut c_void; PROCESS_EVENT_INDEX],
    pub ProcessEvent: unsafe extern "stdcall" fn(
        this: ueptr<UObject>,
        Function: ueptr<UFunction>,
        Parms: Option<NonNull<c_void>>,
        Result: Option<NonNull<c_void>>,
    ),
    _pad_1: [*mut c_void; CALL_FUNCTION_INDEX - PROCESS_EVENT_INDEX],
    pub CallFunction: unsafe extern "stdcall" fn(
        this: ueptr<UObject>,
        TheStack: ueptr<FFrame>,
        Result: Option<NonNull<c_void>>,
        Function: ueptr<UFunction>,
    ),
}

#[repr(C)]
pub struct UObject {
    pub VfTableObject: ueptr<UObjectVtbl>,
    pub HashNext: FPointer,
    pub ObjectFlags: EObjectFlags,
    pub HashOuterNext: FPointer,
    pub StateFrame: FPointer,
    pub Linker: Option<ueptr<UObject>>,
    pub LinkerIndex: FPointer,
    pub ObjectInternalInteger: i32,
    pub NetIndex: i32,
    pub Outer: Option<ueptr<UObject>>,
    pub Name: FName,
    pub Class: ueptr<UClass>,
    pub ObjectArchetype: Option<ueptr<UObject>>,
}

unreal_object!(UObject, "Core", "Object");

impl UObject {
    #[must_use]
    pub fn GObjObjects() -> &'static TArray<Option<ueptr<UObject>>> {
        unsafe { &*crate::globals::gobjects().cast::<TArray<_>>() }
    }

    pub fn ProcessEvent(
        &mut self,
        Function: ueptr<UFunction>,
        Parms: Option<NonNull<c_void>>,
        Result: Option<NonNull<c_void>>,
    ) {
        unsafe {
            ((*self.VfTableObject).ProcessEvent)(
                ueptr(NonNull::from(self)),
                Function,
                Parms,
                Result,
            );
        }
    }

    pub fn CallFunction(
        &mut self,
        TheStack: ueptr<FFrame>,
        Result: Option<NonNull<c_void>>,
        Function: ueptr<UFunction>,
    ) {
        unsafe {
            ((*self.VfTableObject).CallFunction)(
                ueptr(NonNull::from(self)),
                TheStack,
                Result,
                Function,
            );
        }
    }

    pub fn FindClass(FullName: &str) -> Option<ueptr<UClass>> {
        static CLASSES: OnceLock<HashMap<String, i32>> = OnceLock::new();

        let classes = CLASSES.get_or_init(|| {
            UObject::GObjObjects()
                .iter()
                .flatten()
                .filter(|obj| obj.GetFullName().starts_with("Class"))
                .map(|func| (func.GetFullName(), func.ObjectInternalInteger))
                .collect()
        });

        let class = classes
            .get(FullName)
            .map(|&index| UObject::GObjObjects()[index])??;

        Some(ueptr(NonNull::from(&*class).cast()))
    }

    #[must_use]
    pub fn GetName(&self) -> String {
        self.Name.to_string()
    }

    #[must_use]
    pub fn GetFullName(&self) -> String {
        let class = self.Class.GetName();

        let path_name = iter::successors(self.Outer, |outer| outer.Outer)
            .map(|c| c.GetName())
            .fold(self.GetName(), |name, outer| format!("{outer}::{name}"));

        format!("{class} {path_name}")
    }

    #[must_use]
    pub fn GetPathName(&self) -> String {
        if let Some(outer) = self.Outer {
            format!("{}::{}", outer.GetPathName(), self.GetName())
        } else {
            self.GetName()
        }
    }

    #[must_use]
    pub fn IsA<T: UnrealObject>(&self) -> bool {
        let class = T::StaticClass();
        self.Class.iter_superclass().any(|c| c == class)
    }

    #[must_use]
    pub fn Cast<T: UnrealObject>(&self) -> Option<&T> {
        if self.IsA::<T>() {
            Some(unsafe { &*ptr::from_ref(self).cast::<T>() })
        } else {
            None
        }
    }
}
