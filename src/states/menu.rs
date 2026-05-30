use crate::GameState;
use crate::resources::Settings;
use crate::states::editor::Editor;

use egui_macroquad::egui;
use macroquad::miniquad::window::order_quit;

#[derive(Default)]
pub struct Menu {
  should_draw_settings_window: bool,
}

impl Menu {
  pub fn draw_ui(&mut self, egui_ctx: &egui::Context) -> Option<GameState> {
    let inner_response = egui::Window::new("Menu")
      .resizable(false)
      .movable(false)
      .collapsible(false)
      .title_bar(false)
      .show(egui_ctx, |ui| {
        let result = ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
          if ui.button("Start").clicked() {
            todo!()
          }

          if ui.button("Editor").clicked() {
            let editor = Box::new(Editor::new());

            return Some(GameState::Editor(editor));
          }

          if ui.button("Settings").clicked() {
            self.should_draw_settings_window = true;
          }

          if ui.button("Quit").clicked() {
            order_quit();
          }

          None
        });
        result.inner
      })
      .unwrap();

    self.draw_settings_window(egui_ctx);

    // Изменилось ли глобальное-игровое состояние?
    //
    // Если да, то даем основному циклу узнать об этом и предпринять соответствующие меры.
    inner_response.inner.unwrap()
  }

  fn draw_settings_window(&mut self, egui_ctx: &egui::Context) {
    egui::Window::new("Settings")
      .resizable(false)
      .open(&mut self.should_draw_settings_window)
      .show(egui_ctx, |ui| {
        let mut settings = Settings::get_mut();

        ui.add(
          egui::Slider::new(&mut settings.animation_speed_multiplier, 1.0..=5.0)
            .text("Animation speed multiplier"),
        );

        ui.checkbox(&mut settings.show_frames_per_second, "Show FPS");

        if ui.button("Save").clicked() {
          let _ = settings.save();
        }
      });
  }
}
