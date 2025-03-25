use std::ops::Index;
use serde_json::{Result, Value};
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use super::{TRAINER_JSON_STR, TRAINER_POKEMON_JSON_STR, Facility};

#[derive(Serialize, Deserialize, Debug)]
pub struct Jtmon {
  pub id:u16,
  pub species:String,
  #[serde(rename = "displayName")]
  pub display_name:String,
  pub item:String,
  pub speed:u16,
  pub nature:String,
  #[serde(default = "default_nature_id")]
  pub nature_id:u8,
  pub moves:Vec<String>,
  pub abilities:[String;2],

  #[serde(default = "species_formatted")]
  pub species_formatted:String,
}

fn default_nature_id() -> u8 {
  0
}

fn species_formatted() -> String {
  String::from("")
}

impl Jtmon {
  pub fn get_ability_name(&self, ab_idx:usize) -> &str {
    if self.abilities[ab_idx].is_empty() {
      &self.abilities[0]
    } else {
      &self.abilities[ab_idx]
    }
  }
}

pub fn format_name(str:&str) -> String {
  let str:String = str.chars().filter(|c| c.is_alphanumeric()).collect();
  str.to_ascii_lowercase()
}

const NATURES:[&str;25] = ["Hardy","Lonely","Brave","Adamant","Naughty","Bold","Docile","Relaxed","Impish","Lax","Timid","Hasty","Serious","Jolly","Naive","Modest","Mild","Quiet","Bashful","Rash","Calm","Gentle","Sassy","Careful","Quirky"];

#[derive(Serialize, Deserialize, Debug)]
pub struct Jtrainer {
  pub id:u16,
  pub name:String,
  pub pokemons:Vec<u16>,
  #[serde(default = "name_formatted")]
  pub name_formatted:String,
}

fn name_formatted() -> String {
  String::from("")
}

pub static mut TMONS_COMPATIBILITY_MATRIX:[bool;988 * 988] = [false;988 * 988];
pub fn init(){
  console_error_panic_hook::set_once();
  unsafe {

    for tmon1 in JTMONS.iter() {
      for tmon2 in JTMONS.iter() {
        TMONS_COMPATIBILITY_MATRIX[(tmon1.id as usize) * 988 + (tmon2.id as usize)] = tmon1.item != tmon2.item && tmon1.species != tmon2.species;
      }
    }
  }
}


lazy_static! {
  pub static ref JTMONS: Vec<Jtmon> = {
    let mut a: Vec<Jtmon> = serde_json::from_str(&TRAINER_POKEMON_JSON_STR).unwrap();
    for tmon1 in a.iter_mut() {
      tmon1.nature_id = NATURES.iter().position(|&s| s == tmon1.nature).unwrap() as u8;

      tmon1.species_formatted = format_name(&tmon1.species);
    }
    a
  };

  pub static ref JTRAINERS: Vec<Jtrainer> = {
    let mut a: Vec<Jtrainer> = serde_json::from_str(&TRAINER_JSON_STR).unwrap();
    for t in a.iter_mut() {
      //surname dont have é
      t.name_formatted = format_name(&t.name.split(" ").last().unwrap().to_string())
    }
    a
  };

  static ref list0_99:Vec<u16> = Vec::from_iter(0..=99);
  static ref list100_119:Vec<u16> = Vec::from_iter(100..=119);
  static ref list80_119:Vec<u16> = Vec::from_iter(80..=119);
  static ref list120_139:Vec<u16> = Vec::from_iter(120..=139);
  static ref list100_139:Vec<u16> = Vec::from_iter(100..=139);
  static ref list140_159:Vec<u16> = Vec::from_iter(140..=159);
  static ref list120_159:Vec<u16> = Vec::from_iter(120..=159);
  static ref list160_179:Vec<u16> = Vec::from_iter(160..=179);
  static ref list140_179:Vec<u16> = Vec::from_iter(140..=179);
  static ref list180_199:Vec<u16> = Vec::from_iter(180..=199);
  static ref list160_199:Vec<u16> = Vec::from_iter(160..=199);
  static ref list200_219:Vec<u16> = Vec::from_iter(200..=219);
  static ref list180_219:Vec<u16> = Vec::from_iter(180..=219);
  static ref list220_239:Vec<u16> = Vec::from_iter(220..=239);
  static ref list200_299:Vec<u16> = Vec::from_iter(200..=299);
  static ref list300:Vec<u16> = Vec::from_iter(300..=300);
  static ref list301:Vec<u16> = Vec::from_iter(301..=301);
}

pub fn getPossibleTrainersByWins<const F:u32>(wins:u32, trainerIdxInBattle:u32) -> &'static Vec<u16> {
  match F {
    Facility::single | Facility::double => {
      if wins >= 0 && wins < 6 { &list0_99 }
      else if wins == 6 { &list100_119 }
      else if wins >= 7 && wins < 13 { &list80_119 }
      else if wins == 13 { &list120_139 }
      else if wins >= 14 && wins < 20 { &list100_139 }
      else if wins == 20 { &list300 }
      else if wins >= 21 && wins < 27 { &list120_159 }
      else if wins == 27 { &list160_179 }
      else if wins >= 28 && wins < 34 { &list140_179 }
      else if wins == 34 { &list180_199 }
      else if wins >= 35 && wins < 41 { &list160_199 }
      else if wins == 41 { &list200_219 }
      else if wins >= 42 && wins < 48 { &list180_219 }
      else if wins == 48 { &list301 }
      else if wins >= 49 { &list200_299 }
      else { panic!() }
    },
    _ => panic!()
  }
}
