use crate::ArgError;
use crate::sys::*;

use super::FromBoltValue;
use super::MakeBoltValue;
use super::ScalarTypeSignature;
use super::bt_object::Type;

macro_rules! impl_scalar {
    ($t:ty, $id:ident, $expect:ident, $type_fn:path, $get_fn:expr, $check_fn:ident, $make_fn:expr) => {
        impl ScalarTypeSignature for $t {
            fn make_type(ctx: &mut crate::BoltContext) -> Type {
                unsafe {
                    let type_ptr = $type_fn(ctx.as_ptr());
                    Type::from_raw(type_ptr).expect(concat!(
                        "Failed to get ",
                        stringify!($id),
                        " type"
                    ))
                }
            }
        }

        impl FromBoltValue for $t {
            fn from(val: bt_Value) -> Result<Self, ArgError> {
                unsafe {
                    if $check_fn(val) != 0 {
                        Ok($get_fn(val))
                    } else {
                        Err(ArgError::TypeGuard {
                            expected: crate::bt_value::ValueType::$expect,
                            actual: crate::bt_value::ValueType::None,
                        })
                    }
                }
            }

            unsafe fn from_unchecked(val: bt_Value) -> Self {
                unsafe { $get_fn(val) }
            }
        }

        impl MakeBoltValue for $t {
            fn make(&self) -> bt_Value {
                unsafe { $make_fn(*self as _) }
            }
        }
    };
}

impl_scalar!(
    (),
    null,
    Null,
    crate::sys::bt_type_null,
    |_| (),
    bt_is_null,
    |()| bt_make_null()
);

impl_scalar!(
    f64,
    number,
    Number,
    crate::sys::bt_type_number,
    bt_get_number,
    bt_is_number,
    bt_make_number
);

impl_scalar!(
    bool,
    bool,
    Bool,
    crate::sys::bt_type_bool,
    |val| bt_get_bool(val) != 0,
    bt_is_bool,
    bt_make_bool
);
