use crate::components::*;
use crate::serialize::WorldInfo;
use crate::states::menu::Menu;
use crate::{Direction, Game, GameState, WorldGrid, scripting, utils};

use crate::systems::draw::*;
use crate::systems::tick::*;

use egui_macroquad::egui;
use mlua::Lua;

use macroquad::logging as log;
use macroquad::prelude::*;

use std::fs;

pub struct Gameplay {
  pub world_grid: WorldGrid,
  script_path: Option<String>,
  player_entity: Option<hecs::Entity>,
  tick_state: TickState,
  abyss: Abyss,
}

impl Gameplay {
  pub fn new(info: WorldInfo, world: hecs::World) -> Self {
    let mut world_grid = WorldGrid::new(&info, world);

    let player_entity = world_grid
      .query_mut::<(&Player, hecs::Entity)>()
      .into_iter()
      .map(|(_, entity)| entity)
      .next();

    Self {
      world_grid,
      script_path: None,
      player_entity,
      tick_state: TickState::ProcessingLogic,
      abyss: Abyss::default(),
    }
  }

  pub fn draw_ui(&mut self, egui_ctx: &egui::Context) -> Option<GameState> {
    egui::Window::new("Gameplay")
      .resizable(false)
      .show(egui_ctx, |ui| {
        if ui.button("Return to main menu").clicked() {
          let menu = Menu::default();

          return Some(GameState::Menu(menu));
        }

        ui.separator();

        let selected_text = format!("{:?}", self.script_path);

        egui::ComboBox::from_label("Script").selected_text(selected_text).show_ui(ui, |ui| {
          utils::with_entries_in("scripts/", |path, filename| {
            ui.selectable_value(&mut self.script_path, Some(path), filename);
          })
        });

        None
      })
      .and_then(|resp| resp.inner)
      .unwrap()
  }

  pub fn draw(&self, state: &Game) {
    state.with_camera(|state| {
      draw_sprites(&self.world_grid, &state.asset_manager);
    });
  }

  pub fn update(&mut self, lua: &Lua) -> mlua::Result<()> {
    update_sprites(&self.world_grid);

    match self.tick_state {
      TickState::ProcessingLogic => {
        self.update_lua(lua)?;
        self.do_logical_tick();

        self.tick_state = TickState::WaitingForAction;
      }
      TickState::WaitingForAction => {
        if self.update_input() && self.process_actions() {
          self.tick_state = TickState::Animating;
        }
      }
      TickState::Animating => {
        if is_any_animation_active(&self.world_grid) {
          update_animations(&mut self.world_grid);
        } else {
          self.tick_state = TickState::ProcessingLogic;
        }
      }
    }
    Ok(())
  }

  pub fn push_player_action(&mut self, action_kind: ActionKind) {
    let Some(entity) = self.player_entity else {
      return;
    };

    if let Ok(mut action_queue) = self.world_grid.get::<&mut ActionQueue>(entity) {
      action_queue.push_back(action_kind);
    }
  }

  fn do_logical_tick(&mut self) {
    // Тут можно (и нужно) обновлять логическое состояние мира:
    // * Нажимные плиты
    // * Враги
    // * И т.д.

    update_tickable(&mut self.world_grid);
    update_death_causers(&mut self.world_grid);
  }

  fn update_lua(&mut self, lua: &Lua) -> mlua::Result<()> {
    let Some(ref path) = self.script_path else { return Ok(()) };

    match fs::read(path) {
      Ok(bytes) => {
        lua.load(bytes).exec()?;

        scripting::api::on_abyss_call(lua, &mut self.abyss)?;
      }
      Err(err) => log::error!("Failed to read currently selected script: {}", err),
    }
    Ok(())
  }

  fn process_actions(&mut self) -> bool {
    let mut actions = Vec::new();

    for (queue, entity) in self.world_grid.query::<(&mut ActionQueue, hecs::Entity)>().iter() {
      if let Some(action_kind) = queue.pop_front() {
        actions.push((action_kind, entity));
      }
    }

    if actions.is_empty() {
      return false;
    }

    for (action_kind, entity) in actions {
      match action_kind {
        ActionKind::Move(opts) => self.world_grid.move_entity(entity, opts),
        ActionKind::Interact(dir) => self.world_grid.interact(entity, dir),
      }
    }
    true
  }

  fn update_input(&mut self) -> bool {
    let Some(key_pressed) = get_last_key_pressed() else {
      return false;
    };

    if is_any_animation_active(&self.world_grid) {
      return false;
    }

    let move_dir = match key_pressed {
      KeyCode::W => Some(Direction::North),
      KeyCode::A => Some(Direction::West),
      KeyCode::S => Some(Direction::South),
      KeyCode::D => Some(Direction::East),
      _ => None,
    };

    if let Some(dir) = move_dir {
      self.push_player_action(ActionKind::Move(MoveOptions {
        dir,
        can_push: true,
        despawn_if_collided: false,
      }));
    }
    true
  }
}

#[derive(Default)]
pub struct Abyss {}

#[derive(Debug)]
enum TickState {
  ProcessingLogic,
  WaitingForAction,
  Animating,
}
