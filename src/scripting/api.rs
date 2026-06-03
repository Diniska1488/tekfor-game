use crate::states::gameplay::Abyss;

use mlua::prelude::*;

impl LuaUserData for Abyss {}

pub fn on_abyss_call(lua: &Lua, abyss: &mut Abyss) -> LuaResult<()> {
  lua.scope(|scope| {
    let data = scope.create_userdata_ref_mut(abyss)?;

    super::engine::call_func(lua, "on_abyss_call", data)
  })
}
