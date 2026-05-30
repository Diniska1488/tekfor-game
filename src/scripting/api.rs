use crate::components::ActionKind;
use crate::states::gameplay::Gameplay;

use mlua::prelude::*;

pub(super) fn move_player(lua: &Lua, state: &mut Gameplay, dir: LuaValue) -> LuaResult<()> {
  state.push_player_action(ActionKind::Move(lua.from_value(dir)?));

  Ok(())
}

pub(super) fn interact(lua: &Lua, state: &mut Gameplay, dir: LuaValue) -> LuaResult<()> {
  state.push_player_action(ActionKind::Interact(lua.from_value(dir)?));

  Ok(())
}

pub(super) fn wait(_: &Lua, state: &mut Gameplay, _: ()) -> LuaResult<()> {
  state.push_player_action(ActionKind::NoOp);

  Ok(())
}
