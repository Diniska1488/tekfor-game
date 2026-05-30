use crate::components::*;
use crate::resources::AssetID;
use crate::serialize::*;
use crate::states::menu::Menu;
use crate::systems::draw::draw_sprites;
use crate::{Direction, Game, GameState, Grid};

use egui_macroquad::egui;
use strum::IntoEnumIterator;

use macroquad::logging as log;
use macroquad::prelude::*;

use std::fs;
use std::ops::{Deref, DerefMut};

type ComponentAdder = Box<dyn Fn(&mut hecs::EntityBuilder)>;

pub struct Editor {
  level_path: String,
  world: hecs::World,
  component_id: ComponentID,
  component_adders: Vec<ComponentAdder>,
  should_add_component: bool,
  inner_state: InnerState,
}

pub struct InnerState {
  cursor_pos: UVec2,
  selected_entity: Option<hecs::Entity>,
  facing_dir: Direction,
  asset_id: AssetID,
  z_index: u32,
  stateful_object_kind: StatefulObjectKind,
  interactable_handler_kind: InteractableHandlerKind,
}

impl Default for InnerState {
  fn default() -> Self {
    Self {
      cursor_pos: UVec2::ZERO,
      selected_entity: None,
      facing_dir: Direction::North,
      asset_id: AssetID::Dummy,
      z_index: 0,
      stateful_object_kind: StatefulObjectKind::Door,
      interactable_handler_kind: InteractableHandlerKind::Door,
    }
  }
}

impl Deref for Editor {
  type Target = InnerState;

  fn deref(&self) -> &Self::Target {
    &self.inner_state
  }
}

impl DerefMut for Editor {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner_state
  }
}

impl Editor {
  pub fn new() -> Self {
    Self {
      level_path: String::new(),
      world: hecs::World::new(),
      component_id: ComponentID::Sprite,
      component_adders: Vec::new(),
      should_add_component: false,
      inner_state: InnerState::default(),
    }
  }

  pub fn cursor_pos(&self) -> UVec2 {
    self.cursor_pos
  }

  pub fn draw(&self, state: &Game) {
    state.with_camera(None, |state| {
      draw_sprites(&self.world, &state.asset_manager);

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
            let bytes = serialize_as_binary(&self.world)?;

            fs::write(&self.level_path, bytes)?;
          }

          if ui.button("Load").clicked() {
            let bytes = fs::read(&self.level_path)?;

            self.world = deserialize_from_binary(&bytes)?;
          }
          Ok::<(), anyhow::Error>(())
        });

        if let Err(err) = save_load_result.inner {
          log::error!("{}", err);
        }

        let selected_text: &'static str = self.component_id.into();

        let available_components =
          ComponentID::iter().filter(|comp_id| is_available_in_level_editor(*comp_id));

        ui.horizontal(|ui| {
          egui::ComboBox::from_label("Component").selected_text(selected_text).show_ui(ui, |ui| {
            for comp_id in available_components {
              let text: &'static str = comp_id.into();

              ui.selectable_value(&mut self.component_id, comp_id, text);
            }
          });

          self.should_add_component = ui.button("Add").clicked();
        });

        match self.component_id {
          ComponentID::Interactable | ComponentID::Tickable => self.interactable_ui(ui),
          ComponentID::Sprite => self.sprite_ui(ui),
          ComponentID::StatefulObjectKind => self.stateful_ui(ui),
          ComponentID::ZIndex => self.z_index_ui(ui),
          ComponentID::Facing => self.facing_ui(ui),
          ComponentID::Pushable => self.try_add_component(Pushable),
          ComponentID::Movable => self.try_add_component(Movable),
          ComponentID::Closed => self.try_add_component(Closed),
          ComponentID::Solid => self.try_add_component(Solid),
          _ => (),
        };

        if !self.component_adders.is_empty() && ui.button("Spawn entity").clicked() {
          let mut builder = hecs::EntityBuilder::new();

          for adder in self.component_adders.iter() {
            adder(&mut builder);
          }

          builder.add_bundle((Position(self.cursor_pos), OnGrid));

          let entity = self.world.spawn(builder.build());

          log::debug!("Spawned entity via level editor: {:?}", entity);

          self.component_adders.clear();
        }

        ui.label(format!("Position: x: {}, y: {}", self.cursor_pos.x, self.cursor_pos.y));

        None
      })
      .unwrap();

    inner_response.inner.unwrap_or(None)
  }

  pub fn update(&mut self, ui_wants_input: bool) {
    if ui_wants_input {
      return;
    }

    self.update_input();
  }

  fn draw_cursor(&self) {
    let x = self.cursor_pos.x as f32 * Grid::CELL_SIZE;
    let y = self.cursor_pos.y as f32 * Grid::CELL_SIZE;

    draw_rectangle_lines(x, y, Grid::CELL_SIZE, Grid::CELL_SIZE, 2.0, WHITE);
  }

  fn update_input(&mut self) {
    let Some(key_pressed) = get_last_key_pressed() else {
      return;
    };

    let dir = match key_pressed {
      KeyCode::W => Direction::North,
      KeyCode::A => Direction::West,
      KeyCode::S => Direction::South,
      KeyCode::D => Direction::East,
      _ => return,
    };

    self.cursor_pos = crate::utils::advance_pos_in_direction(self.cursor_pos, dir);
  }

  fn interactable_ui(&mut self, ui: &mut egui::Ui) {
    let selected_text = format!("{:?}", self.selected_entity);

    let cursor_pos = self.cursor_pos;

    let entities_under_cursor: Vec<hecs::Entity> = self
      .world
      .query_mut::<(&Position, hecs::Entity)>()
      .into_iter()
      .filter_map(|(pos, ent)| (pos.into_inner() == cursor_pos).then_some(ent))
      .collect();

    egui::ComboBox::from_label("Entity to link").selected_text(selected_text).show_ui(ui, |ui| {
      ui.selectable_value(&mut self.selected_entity, None, "None");

      for entity in entities_under_cursor {
        let text = format!("{:?}", entity);

        ui.selectable_value(&mut self.selected_entity, Some(entity), text);
      }
    });

    let selected_text: &'static str = self.interactable_handler_kind.into();

    egui::ComboBox::from_label("Handler kind").selected_text(selected_text).show_ui(ui, |ui| {
      for kind in InteractableHandlerKind::iter() {
        let text: &'static str = kind.into();

        ui.selectable_value(&mut self.interactable_handler_kind, kind, text);
      }
    });
  }

  fn sprite_ui(&mut self, ui: &mut egui::Ui) {
    let selected_text: &'static str = self.asset_id.into();

    egui::ComboBox::from_label("Asset ID").selected_text(selected_text).show_ui(ui, |ui| {
      for id in AssetID::iter() {
        let text: &'static str = id.into();

        ui.selectable_value(&mut self.asset_id, id, text);
      }
    });

    self.try_add_component(Sprite(self.asset_id));
  }

  fn stateful_ui(&mut self, ui: &mut egui::Ui) {
    let selected_text: &'static str = self.asset_id.into();

    egui::ComboBox::from_label("Stateful object kind").selected_text(selected_text).show_ui(
      ui,
      |ui| {
        for stateful in StatefulObjectKind::iter() {
          let text: &'static str = stateful.into();

          ui.selectable_value(&mut self.stateful_object_kind, stateful, text);
        }
      },
    );

    self.try_add_component(self.stateful_object_kind);
  }

  fn z_index_ui(&mut self, ui: &mut egui::Ui) {
    ui.add(egui::Slider::new(&mut self.z_index, 0..=100));

    self.try_add_component(ZIndex(self.z_index));
  }

  fn facing_ui(&mut self, ui: &mut egui::Ui) {
    let selected_text: &'static str = self.facing_dir.into();

    egui::ComboBox::from_label("Direction").selected_text(selected_text).show_ui(ui, |ui| {
      for dir in Direction::iter() {
        let text: &'static str = dir.into();

        ui.selectable_value(&mut self.facing_dir, dir, text);
      }
    });

    self.try_add_component(Facing(self.facing_dir));
  }

  fn try_add_component<C: hecs::Component + Clone>(&mut self, component: C) {
    if !self.should_add_component {
      return;
    }

    self.component_adders.push(Box::new(move |builder| {
      builder.add(component.clone());
    }));
  }
}

impl Default for Editor {
  fn default() -> Self {
    Self::new()
  }
}

fn is_available_in_level_editor(comp_id: ComponentID) -> bool {
  matches!(
    comp_id,
    ComponentID::Closed
      | ComponentID::Facing
      | ComponentID::Movable
      | ComponentID::Interactable
      | ComponentID::Player
      | ComponentID::Pushable
      | ComponentID::Solid
      | ComponentID::Sprite
      | ComponentID::Tickable
      | ComponentID::StatefulObjectKind
      | ComponentID::ZIndex
  )
}
