use crate::lock_picking::LockKind;

use macroquad::logging as log;
use serde::Serialize;
use strum::IntoEnumIterator;

use mlua::MaybeSend;
use mlua::prelude::*;

pub fn create() -> LuaResult<Lua> {
  let lua = Lua::new();

  add_enum::<LockKind>(&lua)?;

  add_func(&lua, "whisper_to_abyss", |lua, message: LuaValue| {
    log::info!("{}", lua.from_value::<String>(message)?);

    Ok(())
  })?;

  Ok(lua)
}

pub(super) fn call_func<R: FromLuaMulti>(
  lua: &Lua,
  key: impl IntoLua,
  args: impl IntoLuaMulti,
) -> LuaResult<R> {
  lua.globals().get::<LuaFunction>(key)?.call(args)
}

fn add_func<F, A>(lua: &Lua, key: impl IntoLua, f: F) -> LuaResult<()>
where
  F: Fn(&Lua, A) -> LuaResult<()> + MaybeSend + 'static,
  A: FromLuaMulti,
{
  lua.globals().set(key, lua.create_function(f)?)
}

fn add_enum<E>(lua: &Lua) -> LuaResult<()>
where
  E: IntoEnumIterator + Into<&'static str> + Serialize + 'static,
{
  let enum_table = lua.create_table()?;

  for variant in E::iter() {
    let value = lua.to_value(&variant)?;
    let key: &'static str = variant.into();

    enum_table.set(key, value)?;
  }

  let enum_name = crate::utils::type_name_str::<E>();

  lua.globals().set(enum_name, enum_table)
}
