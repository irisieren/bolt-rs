use std::{collections::HashMap, ffi::CString, fmt::Pointer, ptr::NonNull};

use bt_object::{ObjectType, Type};

use crate::{
    ArgError, BoltContext,
    bt_object::BoltObject,
    sys::{self, bt_Value},
};

pub mod bt_object;
pub mod bt_scalar;

/// non-scalar compound types that need more information to create reflection info
pub trait TypeSignature: Sized {
    fn make_type(&self, ctx: &mut BoltContext) -> Type;
}

/// Scalar and simple types only need to know which type they are to produce reflection info
pub trait ScalarTypeSignature: Sized {
    fn make_type(ctx: &mut BoltContext) -> Type;
}

/// Types which can be extracted from a Bolt type
pub trait FromBoltValue: Sized {
    /// Check the type before unwrapping
    fn from(val: bt_Value) -> Result<Self, ArgError>;
    /// Extract without checking - if you know from context that the
    /// typechecker has already run
    unsafe fn from_unchecked(val: bt_Value) -> Self;
}

/// Types which can be boxed into bolt values for use in function calls and return values
/// without help from the context.
pub trait MakeBoltValue: Sized {
    fn make(&self) -> bt_Value;
}

/// Types which can be boxed into bolt values for use in function calls and return values
/// but that need help from the context.
pub trait MakeBoltValueWithContext: Sized {
    fn make_with_context(&self, ctx: &mut BoltContext) -> bt_Value;
}

/// A wrapper around a bolt object or scalar type
#[derive(Debug, Clone, Copy)]
pub struct Value {
    value: bt_Value,
}

impl Value {
    pub fn from_raw(val: bt_Value) -> Self {
        Self { value: val }
    }

    pub fn as_raw(&self) -> bt_Value {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct CallSignature {
    pub args: Vec<Type>,
    pub return_ty: Type,
}

impl CallSignature {
    fn from_type(t: &Type) -> Self {
        unsafe {
            let sig = &(*t.as_raw()).as_.fn_;

            CallSignature {
                args: std::slice::from_raw_parts(sig.args.elements, sig.args.length as usize)
                    .iter()
                    .map(|&arg| Type::from_raw(arg).unwrap())
                    .collect(),
                return_ty: Type::from_raw(sig.return_type).unwrap(),
            }
        }
    }

    pub fn make_type(&self, ctx: &mut BoltContext) -> Type {
        unsafe {
            let mut arg_ptrs: Vec<_> = self.args.iter().map(|t| t.as_raw()).collect();

            let type_ptr = sys::bt_make_signature_type(
                ctx.as_ptr(),
                self.return_ty.as_raw(),
                arg_ptrs.as_mut_ptr(),
                self.args.len() as u8,
            );

            Type::from_raw(type_ptr).expect("Failed to create signature type")
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoltEnum {
    name: CString,
    is_closed: bool,
}

#[derive(Debug, Clone)]
pub enum ValueType {
    Null,
    Bool,
    Number,
    Enum(u32),
    /// None Object - distinct from Null
    None,
    /// Type Signature Object
    Type,
    String,
    Module,
    Import,
    Function(CallSignature),
    NativeFunction(CallSignature),
    Closure(CallSignature),
    Array(Vec<ValueType>),
    Table(HashMap<String, ValueType>),
    UserData,
    Annotation,
}

impl ValueType {
    fn from_object(val: bt_Value) -> Self {
        unsafe {
            if let Some(mut obj) = NonNull::new(sys::bt_object(val)) {
                match BoltObject::from_raw(obj.as_mut()).unwrap() {
                    BoltObject::None => ValueType::None,
                    BoltObject::Type(_) => ValueType::Type,
                    BoltObject::String(_) => ValueType::String,
                    BoltObject::Module(_) => ValueType::Module,
                    BoltObject::Import(_) => ValueType::Import,
                    BoltObject::Function(f) => f.signature.map_or(ValueType::None, |mut cs| {
                        ValueType::Function(CallSignature::from_type(
                            &Type::from_raw(cs.as_mut()).unwrap(),
                        ))
                    }),
                    BoltObject::NativeFunction(_) => ValueType::NativeFunction(todo!()),
                    BoltObject::Closure(c) => c.func.map_or(ValueType::None, |mut f| {
                        ValueType::Function(CallSignature::from_type(
                            &Type::from_raw((*f.as_ptr()).signature).unwrap(),
                        ))
                    }),
                    BoltObject::Array(arr) => ValueType::Array(arr.items),
                    BoltObject::Table(_) => todo!(),
                    BoltObject::UserData(_) => ValueType::UserData,
                    BoltObject::Annotation(_) => ValueType::Annotation,
                }
            } else {
                ValueType::None
            }
        }
    }

    pub fn from_value(val: bt_Value) -> Self {
        unsafe {
            match () {
                _ if crate::sys::bt_is_null(val) != 0 => ValueType::Null,
                _ if crate::sys::bt_is_bool(val) != 0 => ValueType::Bool,
                _ if crate::sys::bt_is_number(val) != 0 => ValueType::Number,
                _ if crate::sys::bt_is_enum_val(val) != 0 => {
                    ValueType::Enum(crate::sys::bt_get_enum_val(val))
                }
                _ if crate::sys::bt_is_object(val) != 0 => Self::from_object(val),
                _ => ValueType::None,
            }
        }
    }
}
