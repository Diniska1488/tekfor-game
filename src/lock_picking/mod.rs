pub mod basic;

use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoStaticStr};

#[derive(Serialize, Deserialize, EnumIter, IntoStaticStr, Clone, Copy, PartialEq)]
pub enum LockKind {
  Basic,
}

// NOTE: Временное решение. Может не подойти для любого другого варианта кроме базового.
pub type LockData<'a> = &'a str;
