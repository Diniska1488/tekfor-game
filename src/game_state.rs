use macroquad::logging as log;
use macroquad::math::{Vec2, vec2};
use macroquad::texture::Texture2D;

use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoStaticStr};

use std::ops::{Deref, DerefMut};

pub struct State {
  pub grid: Grid,
  pub world: hecs::World,
  player_entity: Option<hecs::Entity>,
}

impl State {
  pub fn with_grid_size(width: u32, height: u32) -> Self {
    let grid = Grid::new(width, height);
    let world = hecs::World::new();

    Self { grid, world, player_entity: None }
  }

  pub fn move_entity(&mut self, entity: hecs::Entity, x: u32, y: u32) {
    type MovableGridEntity<'a> = (&'a mut Position, &'a Movable, &'a OnGrid);

    if let Ok((pos, _, _)) = self.world.query_one_mut::<MovableGridEntity>(entity) {
      self.grid.remove_from_cell(pos.x as u32, pos.y as u32, entity);

      pos.x = x as f32;
      pos.y = y as f32;

      self.grid.add_to_cell(x, y, entity);
    }
  }

  pub fn spawn_entity(&mut self, components: impl hecs::DynamicBundle) -> hecs::Entity {
    let entity = self.world.spawn(components);

    if let Ok((pos, _)) = self.world.query_one::<(&Position, &OnGrid)>(entity).get() {
      self.grid.add_to_cell(pos.x as u32, pos.y as u32, entity);
    }
    entity
  }

  pub fn spawn_player(&mut self, components: impl hecs::DynamicBundle) -> hecs::Entity {
    let entity = self.spawn_entity(components);

    self.world.insert_one(entity, PlayerTag).unwrap();
    self.player_entity.replace(entity);

    entity
  }
}

// API
impl State {
  pub fn move_player(&mut self, dir: Direction) {
    log::debug!("Called move_player with {:?} as a parameter", dir);

    let Some(player_entity) = self.player_entity else {
      return;
    };

    let Ok((pos_x, pos_y)) = self
      .world
      .query_one::<(&Position, &Movable, &OnGrid)>(player_entity)
      .get()
      .map(|(pos, _, _)| (pos.x as u32, pos.y as u32))
    else {
      return;
    };

    let (dest_x, dest_y) = match dir {
      Direction::North => (None, Some(pos_y.saturating_sub(1))),
      Direction::East => (Some(pos_x.saturating_add(1)), None),
      Direction::South => (None, Some(pos_y.saturating_add(1))),
      Direction::West => (Some(pos_x.saturating_sub(1)), None),
    };

    let new_pos_x = dest_x.unwrap_or(pos_x);
    let new_pos_y = dest_y.unwrap_or(pos_y);

    self.move_entity(player_entity, new_pos_x, new_pos_y);
  }
}

#[derive(Serialize, Deserialize, EnumIter, IntoStaticStr, Clone, Copy, Debug)]
pub enum Direction {
  North,
  East,
  South,
  West,
}

pub struct Grid {
  cells: Vec<Vec<hecs::Entity>>,
  width: u32,
  height: u32,
}

impl Grid {
  pub const CELL_SIZE: f32 = 32.0;

  pub fn new(width: u32, height: u32) -> Self {
    let capacity = (width * height) as usize;
    let mut cells = Vec::with_capacity(capacity);

    for _ in 0..capacity {
      cells.push(Vec::with_capacity(1));
    }

    Self { cells, width, height }
  }

  pub fn width(&self) -> u32 {
    self.width
  }

  pub fn height(&self) -> u32 {
    self.height
  }

  fn index(&self, x: u32, y: u32) -> Option<usize> {
    if x >= self.width || y >= self.height {
      return None;
    }
    Some((y * self.width + x) as usize)
  }

  fn add_to_cell(&mut self, x: u32, y: u32, entity: hecs::Entity) {
    if let Some(idx) = self.index(x, y) {
      self.cells[idx].push(entity);
    }
  }

  fn remove_from_cell(&mut self, x: u32, y: u32, entity: hecs::Entity) {
    if let Some(idx) = self.index(x, y) {
      self.cells[idx].retain(|&e| e != entity);
    }
  }
}

macro_rules! deref {
  ($from:tt, $into:tt) => {
    impl DerefMut for $from {
      fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
      }
    }

    impl Deref for $from {
      type Target = $into;

      fn deref(&self) -> &Self::Target {
        &self.0
      }
    }

    impl $from {
      #[allow(dead_code)]
      pub fn into_inner(self) -> $into {
        self.0
      }
    }
  };
}

#[derive(Clone, Copy)]
pub struct Position(pub Vec2);

impl Position {
  pub fn global(self) -> Vec2 {
    vec2(self.0.x * Grid::CELL_SIZE, self.0.y * Grid::CELL_SIZE)
  }
}

#[derive(Clone, Copy)]
pub struct ZoomFactor(pub f32);

#[derive(Clone)]
pub struct Sprite(pub Texture2D);

pub struct Movable;
pub struct OnGrid;

struct PlayerTag;

deref!(Position, Vec2);
deref!(ZoomFactor, f32);
deref!(Sprite, Texture2D);
