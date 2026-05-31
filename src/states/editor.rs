use crate::components::*;
use crate::resources::AssetID;
use crate::serialize::*;
use crate::states::menu::Menu;
use crate::systems::draw::draw_sprites;
use crate::{Direction, Game, GameState, Grid, WorldGrid, utils};

use egui_macroquad::egui;
use strum::IntoEnumIterator;

use macroquad::logging as log;
use macroquad::prelude::*;

use std::fs;

type ComponentAdder = Box<dyn Fn(&mut hecs::World, hecs::Entity)>;

pub struct Editor {
  level_path: String,
  world_grid: WorldGrid,
  cursor_pos: UVec2,
  selected_entity: Option<hecs::Entity>,
  component_id: Option<ComponentID>,
  component_adder: Option<ComponentAdder>,
  should_add_component: bool,
  allow_ui_to_capture_keyboard: bool,
  is_in_linkage_mode: bool,
  asset_id: AssetID,
  /// Информация о компонентах, которая будет затем использоваться для конструирования компонента.
  component_info: ComponentInfo,
}

impl Editor {
  pub fn new() -> Self {
    Self {
      level_path: String::new(),
      world_grid: WorldGrid::default(),
      cursor_pos: UVec2::ZERO,
      selected_entity: None,
      component_id: None,
      component_adder: None,
      should_add_component: false,
      allow_ui_to_capture_keyboard: false,
      is_in_linkage_mode: false,
      asset_id: AssetID::Dummy,
      component_info: ComponentInfo::default(),
    }
  }

  pub fn draw(&self, state: &Game) {
    state.with_camera(None, |state| {
      draw_sprites(&self.world_grid, &state.asset_manager);

      self.draw_cursor();
    });
  }

  pub fn draw_ui(&mut self, egui_ctx: &egui::Context) -> Option<GameState> {
    let inner_response = egui::Window::new("Level editor")
      .resizable(false)
      .show(egui_ctx, |ui| {
        if ui.button("Return to main menu").clicked() {
          let menu = Menu::default();

          return Some(GameState::Menu(menu));
        }

        ui.separator();

        let save_load_result = ui.horizontal(|ui| {
          egui::TextEdit::singleline(&mut self.level_path).hint_text("Level path").show(ui);

          if ui.button("Save").clicked() {
            let bytes = serialize_as_binary(&self.world_grid)?;

            fs::write(&self.level_path, bytes)?;
          }

          if ui.button("Load").clicked() {
            let bytes = fs::read(&self.level_path)?;

            let world = deserialize_from_binary(&bytes)?;
            self.world_grid = WorldGrid::with_world(world);
          }
          Ok::<(), anyhow::Error>(())
        });

        if let Err(err) = save_load_result.inner {
          log::error!("{}", err);
        }

        self.draw_current_entity_ui(ui);
        self.draw_component_ui(ui);
        self.draw_asset_ui(ui);

        ui.separator();

        ui.label(format!("Position: x: {}, y: {}", self.cursor_pos.x, self.cursor_pos.y));
        ui.checkbox(&mut self.allow_ui_to_capture_keyboard, "Allow ui to capture keyboard")
          .on_hover_text("Might be useful when is tired of moving cursor away from UI");

        None
      })
      .unwrap();

    inner_response.inner.unwrap_or(None)
  }

  pub fn update(&mut self, ui_wants_input: bool) {
    self.try_add_component();

    if ui_wants_input && self.allow_ui_to_capture_keyboard {
      return;
    }

    self.update_input();
  }

  fn update_input(&mut self) {
    let Some(key_pressed) = get_last_key_pressed() else {
      return;
    };

    if key_pressed == KeyCode::Backspace {
      self.try_despawn_entity_under_cursor()
    }

    let dir = match key_pressed {
      KeyCode::W => Direction::North,
      KeyCode::A => Direction::West,
      KeyCode::S => Direction::South,
      KeyCode::D => Direction::East,
      _ => return,
    };

    self.cursor_pos = crate::utils::advance_pos_in_direction(self.cursor_pos, dir);
    self.selected_entity = self.last_entity_under_cursor();
  }

  fn last_entity_under_cursor(&self) -> Option<hecs::Entity> {
    let cell_entities = self.world_grid.get_cell(self.cursor_pos.x, self.cursor_pos.y)?;
    cell_entities.last().copied()
  }

  fn try_add_component(&mut self) {
    if self.component_id.is_none() || !self.should_add_component {
      return;
    }

    if let Some((entity, comp)) = self.selected_entity.zip(self.component_adder.take()) {
      comp(&mut self.world_grid, entity);
    }
  }

  fn try_despawn_entity_under_cursor(&mut self) {
    if let Some(entity) = self.selected_entity {
      let _ = self.world_grid.despawn_entity(entity);

      self.selected_entity = self.last_entity_under_cursor();
    }
  }

  fn draw_cursor(&self) {
    let x = self.cursor_pos.x as f32 * Grid::CELL_SIZE;
    let y = self.cursor_pos.y as f32 * Grid::CELL_SIZE;

    let color = if self.is_in_linkage_mode { GREEN } else { WHITE };

    draw_rectangle_lines(x, y, Grid::CELL_SIZE, Grid::CELL_SIZE, 2.0, color);
  }

  fn draw_current_entity_ui(&mut self, ui: &mut egui::Ui) {
    let Some(cell_entities) = self.world_grid.get_cell(self.cursor_pos.x, self.cursor_pos.y) else {
      return;
    };

    let selected_text: &'static str = self
      .selected_entity
      .and_then(|entity| utils::entity_sprite_text(&self.world_grid, entity))
      .unwrap_or("...");

    egui::ComboBox::from_label("Current entity").selected_text(selected_text).show_ui(ui, |ui| {
      for &entity in cell_entities {
        let Some(text) = utils::entity_sprite_text(&self.world_grid, entity) else {
          continue;
        };

        let entity_mut_ref = match self.is_in_linkage_mode {
          true => &mut self.component_info.linked_entity,
          false => &mut self.selected_entity,
        };

        ui.selectable_value(entity_mut_ref, Some(entity), text);
      }
    });
  }

  fn draw_component_ui(&mut self, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
      let selected_text: &'static str = self.component_id.map(Into::into).unwrap_or("...");

      egui::ComboBox::from_label("Component").selected_text(selected_text).show_ui(ui, |ui| {
        ui.selectable_value(
          &mut self.component_id,
          Some(ComponentID::Interactable),
          "Interactable",
        );
        ui.selectable_value(&mut self.component_id, Some(ComponentID::Tickable), "Tickable");
        ui.selectable_value(&mut self.component_id, Some(ComponentID::ZIndex), "Z index");
        ui.selectable_value(&mut self.component_id, Some(ComponentID::Facing), "Facing");
      });

      if self.component_id.is_some() && self.selected_entity.is_some() {
        self.should_add_component = ui.button("Add").clicked();
      }
    });

    let Some(comp_id) = self.component_id else {
      return;
    };

    match comp_id {
      ComponentID::ZIndex => self.draw_z_index_ui(ui),
      ComponentID::Interactable => self.draw_interactable_ui(ui),
      ComponentID::Tickable => self.draw_tickable_ui(ui),
      ComponentID::Facing => self.draw_facing_ui(ui),
      ComponentID::Animation
      | ComponentID::ActionQueue
      | ComponentID::StatefulObjectKind
      | ComponentID::Position
      | ComponentID::Sprite
      | ComponentID::Closed
      | ComponentID::Movable
      | ComponentID::Pushable
      | ComponentID::OnGrid
      | ComponentID::Solid
      | ComponentID::Player => unreachable!(),
    }
  }

  fn draw_asset_ui(&mut self, ui: &mut egui::Ui) {
    let selected_text: &'static str = self.asset_id.into();

    egui::ComboBox::from_label("Asset").selected_text(selected_text).show_ui(ui, |ui| {
      for asset_id in AssetID::iter() {
        let text: &'static str = asset_id.into();

        ui.selectable_value(&mut self.asset_id, asset_id, text);
      }
    });

    ui.horizontal(|ui| {
      if self.asset_id != AssetID::Dummy && ui.button("Spawn entity").clicked() {
        let entity = match self.asset_id {
          AssetID::Player => self.world_grid.spawn_player_at(self.cursor_pos),
          AssetID::DoorClosed => self.world_grid.spawn_door_at(self.cursor_pos, false),
          AssetID::DoorOpen => self.world_grid.spawn_door_at(self.cursor_pos, true),
          wall_asset_id @ (AssetID::WallHorizontal
          | AssetID::WallHorizontalLeftEdge
          | AssetID::WallHorizontalRightEdge
          | AssetID::WallLeftLowerCorner
          | AssetID::WallLeftUpperCorner
          | AssetID::WallRightLowerCorner
          | AssetID::WallRightUpperCorner
          | AssetID::WallVertical) => self.world_grid.spawn_wall_at(self.cursor_pos, wall_asset_id),
          AssetID::PressurePlate => self.world_grid.spawn_pressure_plate(self.cursor_pos),
          AssetID::Crate => self.world_grid.spawn_crate_at(self.cursor_pos),
          AssetID::Dummy => unreachable!(),
        };

        self.selected_entity.replace(entity);
      }

      if self.selected_entity.is_some() && ui.button("Despawn entity").clicked() {
        self.try_despawn_entity_under_cursor();
      }
    });
  }

  fn draw_interactable_tickable_ui(&mut self, ui: &mut egui::Ui) {
    ui.checkbox(&mut self.is_in_linkage_mode, "Linkage mode");

    let entity_text = self
      .component_info
      .linked_entity
      .and_then(|entity| utils::entity_sprite_text(&self.world_grid, entity))
      .unwrap_or("None");

    ui.label(format!("Linked entity: {}", entity_text));

    let selected_text: &'static str =
      self.component_info.interactable_handler_kind.map(Into::into).unwrap_or("...");

    egui::ComboBox::from_label("Handler kind").selected_text(selected_text).show_ui(ui, |ui| {
      for kind in InteractableHandlerKind::iter() {
        let text: &'static str = kind.into();

        ui.selectable_value(&mut self.component_info.interactable_handler_kind, Some(kind), text);
      }
    });
  }

  fn draw_interactable_ui(&mut self, ui: &mut egui::Ui) {
    self.draw_interactable_tickable_ui(ui);

    if let Some(handler_kind) = self.component_info.interactable_handler_kind {
      self.try_update_component_adder(Interactable {
        linked_entity: self.component_info.linked_entity,
        handler_kind,
      });
    }
  }

  fn draw_tickable_ui(&mut self, ui: &mut egui::Ui) {
    self.draw_interactable_tickable_ui(ui);

    if let Some(handler_kind) = self.component_info.interactable_handler_kind {
      self.try_update_component_adder(Tickable(Interactable {
        linked_entity: self.component_info.linked_entity,
        handler_kind,
      }));
    }
  }

  fn draw_z_index_ui(&mut self, ui: &mut egui::Ui) {
    ui.add(egui::Slider::new(&mut self.component_info.z_index, 0..=100));

    self.try_update_component_adder(ZIndex(self.component_info.z_index));
  }

  fn draw_facing_ui(&mut self, ui: &mut egui::Ui) {
    let selected_text: &'static str =
      self.component_info.facing_dir.map(Into::into).unwrap_or("...");

    egui::ComboBox::from_label("Direction").selected_text(selected_text).show_ui(ui, |ui| {
      for dir in Direction::iter() {
        let text: &'static str = dir.into();

        ui.selectable_value(&mut self.component_info.facing_dir, Some(dir), text);
      }
    });

    if let Some(facing_dir) = self.component_info.facing_dir {
      self.try_update_component_adder(Facing(facing_dir));
    }
  }

  fn try_update_component_adder<C: hecs::Component + Clone>(&mut self, component: C) {
    if !self.should_add_component {
      return;
    }

    self.component_adder.replace(Box::new(move |world, entity| {
      let _ = world.insert_one(entity, component.clone());
    }));
  }
}

impl Default for Editor {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Default)]
struct ComponentInfo {
  linked_entity: Option<hecs::Entity>,
  interactable_handler_kind: Option<InteractableHandlerKind>,
  facing_dir: Option<Direction>,
  z_index: u32,
}
