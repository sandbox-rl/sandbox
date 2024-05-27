use std::collections::HashMap;
use std::sync::OnceLock;
use std::{ffi::c_void, ptr::NonNull};
use std::{iter, mem};

use bitflags::bitflags;

use crate::{
    ueptr, FFrame, FName, FPointer, TArray, UClass, UFunction, UnrealObject, CALL_FUNCTION_INDEX,
    PROCESS_EVENT_INDEX,
};

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct EObjectFlags: u64 {
        const NoFlags = 0x00000000;
        const Public = 0x00000001;
        const Standalone = 0x00000002;
        const MarkAsNative = 0x00000004;
        const Transactional = 0x00000008;
        const ClassDefaultObject = 0x00000010;
        const ArchetypeObject = 0x00000020;
        const Transient = 0x00000040;
        const MarkAsRootSet = 0x00000080;
        const TagGarbageTemp = 0x00000100;
        const NeedInitialization = 0x00000200;
        const NeedLoad = 0x00000400;
        const KeepForCooker = 0x00000800;
        const NeedPostLoad = 0x00001000;
        const NeedPostLoadSubobjects = 0x00002000;
        const NewerVersionExists = 0x00004000;
        const BeginDestroyed = 0x00008000;
        const FinishDestroyed = 0x00010000;
        const BeingRegenerated = 0x00020000;
        const DefaultSubObject = 0x00040000;
        const WasLoaded = 0x00080000;
        const TextExportTransient = 0x00100000;
        const LoadCompleted = 0x00200000;
        const InheritableComponentTemplate = 0x00400000;
        const DuplicateTransient = 0x00800000;
        const StrongRefOnFrame = 0x01000000;
        const NonPIEDuplicateTransient = 0x02000000;
        const Dynamic = 0x04000000;
        const WillBeLoaded = 0x08000000;
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
    pub VfTableObject: *mut UObjectVtbl,
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
            )
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
            )
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

    pub fn GetName(&self) -> String {
        self.Name.to_string()
    }

    pub fn GetFullName(&self) -> String {
        let class = self.Class.GetName();

        let path_name = iter::successors(self.Outer, |outer| outer.Outer)
            .map(|c| c.GetName())
            .fold(self.GetName(), |name, outer| format!("{outer}::{name}"));

        format!("{class} {path_name}")
    }

    pub fn GetPathName(&self) -> String {
        if let Some(outer) = self.Outer {
            format!("{}::{}", outer.GetPathName(), self.GetName())
        } else {
            self.GetName()
        }
    }

    pub fn IsA<T: UnrealObject>(&self) -> bool {
        let class = T::StaticClass();
        self.Class.iter_superclass().any(|c| c == class)
    }

    pub fn Cast<T: UnrealObject>(&self) -> Option<&T> {
        if self.IsA::<T>() {
            Some(unsafe { mem::transmute::<&UObject, &T>(self) })
        } else {
            None
        }
    }
}
