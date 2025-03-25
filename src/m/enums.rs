
#[allow(dead_code)]

use serde_json::{Result, Value};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug,Clone,Copy)]
pub enum Game
{
    Diamond,
    Pearl,
    Platinum,
    Black,
    White,
    Black2,
    White2,
}

#[derive(Serialize, Deserialize, Debug,Clone,Copy)]
pub enum Language
{
    English,
    French,
    German,
    Italian,
    Japanese,
    Korean,
    Spanish,
}

#[derive(Serialize, Deserialize, Debug, PartialEq,Clone,Copy)]
pub enum DSType
{
    DSOriginal,
    DSi,
    DS3,
}

#[derive(Serialize, Deserialize, Debug, PartialEq,Clone,Copy)]
pub enum FacilityEnum {
  single,
  double,
}

impl FacilityEnum {
  pub fn tmp(&self) -> u32{
    match self {
      FacilityEnum::single => { Facility::single },
      FacilityEnum::double => { Facility::double },
    }
  }
  pub fn to_str(&self) -> &str{
    match self {
      FacilityEnum::single => { "single" },
      FacilityEnum::double => { "double" },
    }
  }
}

pub struct Facility {}

impl Facility {
  pub const single:u32 = 0;
  pub const double:u32 = 1;

  pub const fn isMulti(v:u32) -> bool {
    match v {
      //Facility::single => true,
      _ => false,
    }
  }
  pub const fn isDouble(v:u32) -> bool {
    match v {
      Facility::double => true,
      _ => false,
    }
  }
  pub const fn isSingle(v:u32) -> bool {
    match v {
      Facility::single => true,
      _ => false,
    }
  }

	pub const fn getPokemonCount(v:u32) -> usize {
    Facility::getPokemonCountByTrainer(v) * Facility::getTrainerCount(v)
	}
	pub const fn getPokemonCountByTrainer(v:u32) -> usize {
		if Facility::isMulti(v) { 2 }
		else if Facility::isSingle(v) { 3 }
		else { 4 }
	}
	pub const fn getTrainerCountByBattle(v:u32) -> usize {
		if Facility::isMulti(v) { 2 } else { 1 }
	}
	pub const fn getTrainerCount(v:u32) -> usize {
		Facility::getTrainerCountByBattle(v) * Facility::getBattleCount(v)
	}
	pub const fn getBattleCount(v:u32) -> usize {
    7
	}
}


pub struct ExecObjective {}

impl ExecObjective {
  pub const generate_one:u32 = 0;
  pub const search_easy:u32 = 1;
  pub const find_seeds:u32 = 2;
}

