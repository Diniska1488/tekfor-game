use serde::Serialize;
use strum::{EnumIter, IntoStaticStr};

#[derive(Serialize, EnumIter, IntoStaticStr)]
pub enum LockPickKind {
  Basic,
}
