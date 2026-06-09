use crate::states::gameplay::Abyss;

use crate::lock_picking::LockKind;
use crate::lock_picking::basic::Basic as BasicLockPick;

use mlua::prelude::*;

impl LuaUserData for Abyss {}

pub fn on_abyss_call(lua: &Lua, abyss: &mut Abyss) -> LuaResult<()> {
  lua.scope(|scope| {
    let data = scope.create_userdata_ref_mut(abyss)?;

    super::engine::call_func(lua, "on_abyss_call", data)
  })
}

pub fn on_lock_pick(lua: &Lua, kind: LockKind) -> LuaResult<bool> {
  Ok(match kind {
    LockKind::Basic => BasicLockPick::default().is_valid_with(|seq| {
      let kind = lua.to_value(&kind)?;
      let data = lua.to_value(seq)?;

      super::engine::call_func::<bool>(lua, "on_lock_pick", (kind, data))
    }),
  })
}
