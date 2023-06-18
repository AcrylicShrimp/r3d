use mlua::prelude::*;
use parking_lot::Mutex;
use smartstring::alias::String as SmartString;
use std::{cell::RefCell, mem::MaybeUninit, rc::Rc, sync::Arc};

pub trait ConversionByValueReadOnly
where
    Self: Sized,
{
    fn perform_convertion_to_lua<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>>;
}

pub trait ConversionByValue
where
    Self: Sized,
{
    fn perform_conversion_from_lua<'lua>(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self>;
}

macro_rules! impl_lua_readonly {
    ($ty:ty) => {
        impl ConversionByValueReadOnly for $ty {
            fn perform_convertion_to_lua<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
                self.clone().to_lua(lua)
            }
        }
    };
}

macro_rules! impl_lua {
    ($ty:ty) => {
        impl_lua_readonly!($ty);

        impl ConversionByValue for $ty {
            fn perform_conversion_from_lua<'lua>(
                value: LuaValue<'lua>,
                lua: &'lua Lua,
            ) -> LuaResult<Self> {
                Self::from_lua(value, lua)
            }
        }
    };
}

impl ConversionByValueReadOnly for () {
    fn perform_convertion_to_lua<'lua>(&self, _lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(LuaValue::Nil)
    }
}

impl ConversionByValue for () {
    fn perform_conversion_from_lua<'lua>(
        value: LuaValue<'lua>,
        _lua: &'lua Lua,
    ) -> LuaResult<Self> {
        match value {
            LuaValue::Nil => Ok(()),
            _ => Err(LuaError::FromLuaConversionError {
                from: "nil",
                to: "unit",
                message: Some(format!("expected a nil, got [{}]", value.type_name())),
            }),
        }
    }
}

impl_lua!(i8);
impl_lua!(i16);
impl_lua!(i32);
impl_lua!(i64);
impl_lua!(i128);
impl_lua!(isize);
impl_lua!(u8);
impl_lua!(u16);
impl_lua!(u32);
impl_lua!(u64);
impl_lua!(u128);
impl_lua!(usize);
impl_lua!(f32);
impl_lua!(f64);

impl ConversionByValueReadOnly for char {
    fn perform_convertion_to_lua<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let mut tmp = [0; 4];
        self.encode_utf8(&mut tmp);
        Ok(LuaValue::String(lua.create_string(&tmp)?))
    }
}

impl ConversionByValue for char {
    fn perform_conversion_from_lua<'lua>(
        value: LuaValue<'lua>,
        _lua: &'lua Lua,
    ) -> LuaResult<Self> {
        match value {
            LuaValue::String(str) => {
                let str = str.to_str()?;
                if str.len() == 1 {
                    Ok(str.chars().next().unwrap())
                } else {
                    Err(LuaError::FromLuaConversionError {
                        from: "string",
                        to: "char",
                        message: Some(format!(
                            "expected a string of single character, got {} character(s)",
                            str.len()
                        )),
                    })
                }
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "string",
                to: "char",
                message: Some(format!("expected a string, got [{}]", value.type_name())),
            }),
        }
    }
}

impl_lua!(bool);

impl<'s> ConversionByValueReadOnly for &'s str {
    fn perform_convertion_to_lua<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(LuaValue::String(lua.create_string(self)?))
    }
}

impl ConversionByValueReadOnly for String {
    fn perform_convertion_to_lua<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(LuaValue::String(lua.create_string(self)?))
    }
}

impl ConversionByValue for String {
    fn perform_conversion_from_lua<'lua>(
        value: LuaValue<'lua>,
        _lua: &'lua Lua,
    ) -> LuaResult<Self> {
        match value {
            LuaValue::String(str) => Ok(Self::from(str.to_str()?)),
            _ => Err(LuaError::FromLuaConversionError {
                from: "string",
                to: "String",
                message: Some(format!("expected a string, got [{}]", value.type_name())),
            }),
        }
    }
}

impl ConversionByValueReadOnly for SmartString {
    fn perform_convertion_to_lua<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(LuaValue::String(lua.create_string(self)?))
    }
}

impl ConversionByValue for SmartString {
    fn perform_conversion_from_lua<'lua>(
        value: LuaValue<'lua>,
        _lua: &'lua Lua,
    ) -> LuaResult<Self> {
        match value {
            LuaValue::String(str) => Ok(Self::from(str.to_str()?)),
            _ => Err(LuaError::FromLuaConversionError {
                from: "string",
                to: "SmartString",
                message: Some(format!("expected a string, got [{}]", value.type_name())),
            }),
        }
    }
}

impl<T, const N: usize> ConversionByValueReadOnly for [T; N]
where
    T: ConversionByValueReadOnly,
{
    fn perform_convertion_to_lua<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let table = lua.create_table()?;
        for (i, value) in self.iter().enumerate() {
            table.set(i + 1, value.perform_convertion_to_lua(lua)?)?;
        }
        Ok(LuaValue::Table(table))
    }
}

impl<T, const N: usize> ConversionByValue for [T; N]
where
    T: ConversionByValue,
{
    fn perform_conversion_from_lua<'lua>(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Table(table) => {
                let mut array: MaybeUninit<[T; N]> = MaybeUninit::uninit();
                let ptr: *mut T = array.as_mut_ptr().cast();
                for i in 0..N as isize {
                    unsafe {
                        let element = ptr.offset(i);
                        element.write(T::perform_conversion_from_lua(table.get(i + 1)?, lua)?);
                    }
                }
                Ok(unsafe { array.assume_init() })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "table",
                to: "[T; N]",
                message: Some(format!("expected a table, got [{}]", value.type_name())),
            }),
        }
    }
}

impl<T> ConversionByValueReadOnly for Box<T>
where
    T: ConversionByValueReadOnly,
{
    fn perform_convertion_to_lua<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        self.as_ref().perform_convertion_to_lua(lua)
    }
}

impl<T> ConversionByValue for Box<T>
where
    T: ConversionByValue,
{
    fn perform_conversion_from_lua<'lua>(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        T::perform_conversion_from_lua(value, lua).map(Self::new)
    }
}

impl<T> ConversionByValueReadOnly for Rc<T>
where
    T: ConversionByValueReadOnly,
{
    fn perform_convertion_to_lua<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        self.as_ref().perform_convertion_to_lua(lua)
    }
}

impl<T> ConversionByValue for Rc<T>
where
    T: ConversionByValue,
{
    fn perform_conversion_from_lua<'lua>(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        T::perform_conversion_from_lua(value, lua).map(Self::new)
    }
}

impl<T> ConversionByValueReadOnly for Arc<T>
where
    T: ConversionByValueReadOnly,
{
    fn perform_convertion_to_lua<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        self.as_ref().perform_convertion_to_lua(lua)
    }
}

impl<T> ConversionByValue for Arc<T>
where
    T: ConversionByValue,
{
    fn perform_conversion_from_lua<'lua>(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        T::perform_conversion_from_lua(value, lua).map(Self::new)
    }
}

impl<T> ConversionByValueReadOnly for RefCell<T>
where
    T: ConversionByValueReadOnly,
{
    fn perform_convertion_to_lua<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        self.borrow().perform_convertion_to_lua(lua)
    }
}

impl<T> ConversionByValue for RefCell<T>
where
    T: ConversionByValue,
{
    fn perform_conversion_from_lua<'lua>(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        T::perform_conversion_from_lua(value, lua).map(Self::new)
    }
}

impl<T> ConversionByValueReadOnly for Mutex<T>
where
    T: ConversionByValueReadOnly,
{
    fn perform_convertion_to_lua<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        self.lock().perform_convertion_to_lua(lua)
    }
}

impl<T> ConversionByValue for Mutex<T>
where
    T: ConversionByValue,
{
    fn perform_conversion_from_lua<'lua>(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        T::perform_conversion_from_lua(value, lua).map(Self::new)
    }
}

impl<T> ConversionByValueReadOnly for Option<T>
where
    T: ConversionByValueReadOnly,
{
    fn perform_convertion_to_lua<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        match self {
            Some(value) => value.perform_convertion_to_lua(lua),
            None => Ok(LuaValue::Nil),
        }
    }
}

impl<T> ConversionByValue for Option<T>
where
    T: ConversionByValue,
{
    fn perform_conversion_from_lua<'lua>(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Nil => Ok(None),
            _ => Ok(Some(T::perform_conversion_from_lua(value, lua)?)),
        }
    }
}
