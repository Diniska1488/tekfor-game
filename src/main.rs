mod game_state;
mod lua_api;

use macroquad::logging as log;
use macroquad::prelude::*;

use macroquad::ui::widgets::Window;
use macroquad::ui::{Skin, Ui, hash, root_ui};

use game_state::*;

#[macroquad::main(window_conf)]
async fn main() -> anyhow::Result<()> {
  let global_ui_skin = ui_skin(&mut root_ui());

  let mut state = State::with_grid_size(3, 3);

  let centered_camera_pos = vec2(state.grid.width() as f32 / 2.0, state.grid.height() as f32 / 2.0);
  let camera_entity =
    state.spawn_entity((Position(centered_camera_pos), ZoomFactor(2.0), CameraTag));

  let player_pos = vec2(0.0, 0.0);
  state.spawn_player((Position(player_pos), Sprite(Texture2D::empty()), Movable, OnGrid));

  let lua = lua_api::create().unwrap();
  // lua_api::run(&lua, &mut state, "move_player(Direction.North)").unwrap();

  loop {
    clear_background(BLUE);

    // Test
    if is_key_pressed(KeyCode::Space) {
      lua_api::run(
        &lua,
        &mut state,
        r#"
          move_player(Direction.South)
          move_player(Direction.East)
        "#,
      )
      .unwrap();
    }

    draw_ui(&global_ui_skin);

    update_camera(&mut state.world, camera_entity);
    let camera = construct_camera(&state.world, camera_entity);

    set_camera(&camera);
    {
      draw_sprites(&state.world);
    }
    set_default_camera();

    next_frame().await;
  }
}

fn window_conf() -> Conf {
  Conf {
    window_title: String::from("Tekfor game"),
    high_dpi: true,
    fullscreen: true,
    ..Default::default()
  }
}

fn update_camera(world: &mut hecs::World, camera_entity: hecs::Entity) {
  if is_ui_active() {
    return;
  }

  let Ok((zoom_factor, camera_pos)) =
    world.query_one_mut::<(&mut ZoomFactor, &mut Position)>(camera_entity)
  else {
    return;
  };

  handle_zoom_factor(zoom_factor, 1.0, 5.0);

  if is_mouse_button_down(MouseButton::Left) {
    let mouse_delta = mouse_delta_position();
    let speed = 15.0 / zoom_factor.into_inner();

    camera_pos.x += mouse_delta.x * speed;
    camera_pos.y += mouse_delta.y * speed;
  }
}

fn handle_zoom_factor(zoom_factor: &mut f32, min: f32, max: f32) {
  let mut factor = *zoom_factor;
  let (_, wheel_y) = mouse_wheel();

  if wheel_y.abs() > 0.01 {
    let speed = 1.0 + (wheel_y.abs() * 0.01);

    if wheel_y > 0.0 {
      factor *= speed;
    } else {
      factor /= speed;
    }
  }

  *zoom_factor = factor.clamp(min, max);
}

fn construct_camera(world: &hecs::World, camera_entity: hecs::Entity) -> Camera2D {
  let zoom_factor = world.get::<&ZoomFactor>(camera_entity).map(|zf| zf.into_inner()).unwrap();

  let display_rect = Rect::new(0.0, 0.0, screen_width(), screen_height());
  let mut camera = Camera2D::from_display_rect(display_rect);

  // TODO: Zoom on player
  let camera_pos = world.get::<&Position>(camera_entity).unwrap();
  camera.target = camera_pos.global();

  camera.zoom.x *= zoom_factor;
  camera.zoom.y *= -zoom_factor;

  camera
}

fn is_ui_active() -> bool {
  root_ui().is_mouse_over(mouse_position().into())
}

fn draw_sprites(world: &hecs::World) {
  for (pos, sprite) in world.query::<(&Position, &Sprite)>().iter() {
    let global_pos = pos.global();

    draw_texture(sprite, global_pos.x, global_pos.y, WHITE);
    draw_rectangle_lines(global_pos.x, global_pos.y, Grid::CELL_SIZE, Grid::CELL_SIZE, 2.0, BLACK);
  }
}

fn draw_ui(skin: &Skin) {
  root_ui().push_skin(skin);

  draw_fps();

  Window::new(hash!(), vec2(470.0, 50.0), vec2(300.0, 300.0)).ui(&mut root_ui(), |ui| {
    ui.label(None, "Test label");

    if ui.button(None, "Test button") {
      log::info!("Test button was pressed");
    }
  });

  root_ui().pop_skin();
}

fn ui_skin(ui: &mut Ui) -> Skin {
  let label_style = ui.style_builder().font_size(48).build();

  let button_style = ui
    .style_builder()
    .font_size(32)
    .color(Color::from_hex(0xDEE2E6))
    .color_hovered(Color::from_hex(0xCED4DA))
    .color_clicked(Color::from_hex(0xADB5BD))
    .margin(RectOffset::new(10.0, 10.0, 10.0, 10.0))
    .build();

  Skin { label_style, button_style, ..ui.default_skin() }
}

struct CameraTag;
