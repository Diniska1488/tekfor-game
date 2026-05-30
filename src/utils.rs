use crate::{Direction, Grid};

use macroquad::math::*;

pub fn global_pos(pos: UVec2) -> Vec2 {
  vec2(pos.x as f32, pos.y as f32) * Grid::CELL_SIZE
}

pub fn advance_pos_in_direction(pos: UVec2, dir: Direction) -> UVec2 {
  let (dest_x, dest_y) = match dir {
    Direction::North => (None, Some(pos.y.saturating_sub(1))),
    Direction::East => (Some(pos.x + 1), None),
    Direction::South => (None, Some(pos.y + 1)),
    Direction::West => (Some(pos.x.saturating_sub(1)), None),
  };

  let new_pos_x = dest_x.unwrap_or(pos.x);
  let new_pos_y = dest_y.unwrap_or(pos.y);

  uvec2(new_pos_x, new_pos_y)
}
