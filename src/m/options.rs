use serde::{Serialize, Deserialize};

use super::{DSType, Game, Language, FacilityEnum, Facility, ExecObjective, Jtmon,printerr,Date,Strainer,Stmon,PMON_JSON_STR,JTMONS,JTRAINERS,format_name, str_to_u32, validate_str_to_u32};

const TMON_COUNT:usize = 956;

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct PteamForMulti {
  pub pokemon1:String,
  pub pokemon2:String,
  pub item1:String,
  pub item2:String,
  pub multiTrainer:u16,              // 300 Attack, 301 Defence, 302 Balanced
}

impl PteamForMulti {
  fn to_string(&self) -> String {
    format!("Trainer:{}, Poke1:{}@{}, Poke2:{}@{}", self.pokemon1, self.item1, self.pokemon2, self.item2, self.multiTrainer)
  }
}

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Pmon {
  pub name:String,
  #[serde(default = "default_id")]
  pub id:u32,
  pub speed:u16,
  pub item:String,
  pub description:String,
  pub moves:[String;4],
  #[serde(default = "default_ratingsLen")]
  pub ratingsLen:usize,
  pub ratingsByMove:Vec<Vec<(u16,u8,f32)>>,
  #[serde(default = "default_ratingsByMovePokemonAndAbility")]
  pub ratingsByMovePokemonAndAbility:Vec<f32>,
  #[serde(default = "default_is_shedinja")]
  pub is_shedinja:bool
}

fn default_ratingsByMovePokemonAndAbility() -> Vec<f32> {
  vec![]
}
fn default_id() -> u32 {
  0
}
fn default_ratingsLen() -> usize {
  0
}
fn default_is_shedinja() -> bool {
  false
}


impl Pmon {
  pub fn init(&mut self){
    self.ratingsLen = self.ratingsByMove.len();
    self.is_shedinja = self.name == "Shedinja";
    self.ratingsByMovePokemonAndAbility.resize(TMON_COUNT*2 * self.ratingsByMove.len(), 0f32);

    for (i,ratings) in self.ratingsByMove.iter().enumerate() {
      for (monId, abId, rating) in ratings.iter(){
        if *abId == 0 || *abId == 2 {
          self.ratingsByMovePokemonAndAbility[*monId as usize * 2 + 0 as usize + i * TMON_COUNT*2] = *rating;
        }
        if *abId == 1 || *abId == 2 {
          self.ratingsByMovePokemonAndAbility[*monId as usize * 2 + 1 as usize + i * TMON_COUNT*2] = *rating;
        }
      }
      // Palmer jmon are always rating=1 toavoid filter issues
      for monId in 950..=955 {
        for abId in 0..=1 {
          self.ratingsByMovePokemonAndAbility[monId as usize * 2 + abId as usize + i * TMON_COUNT*2] = 1f32;
        }
      }
    }
  }
  pub fn getPokemonRating(&self, trainerPokemon:&Stmon, move_idx:usize) -> f32 {
    unsafe { *self.ratingsByMovePokemonAndAbility.get_unchecked(trainerPokemon.id as usize * 2 + trainerPokemon.ability as usize + move_idx * TMON_COUNT*2) }
  }
  pub fn is_shedinja(&self) -> bool {
    self.id == 0
  }
}

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Options {
  pub errors:Vec<String>,
  pub exec_obj:u32,
  pub wins:u32,
  pub facility:FacilityEnum,
  pub print_rng_frames_info:bool,

  pub same_day_seed:Option<String>,
  pub diff_day_seed:Option<String>,

  /// search multi
  pub filter_min_rating:f32,
  pub player_pokemons:Vec<Pmon>,
  pub max_diff_day_adv:u32,
  pub max_same_day_adv:u32,
  pub at_least_one_same_day_adv:bool,
  pub max_threads:Option<u64>,
  pub player_pokemons_idx_filter:Option<String>,
  pub search_stop_on_perfect_rating:bool,
  pub search_update_min_filter:bool,
  /// multi
  pub player_team_for_multi:Option<PteamForMulti>,
  // find_seed
  pub battled_pokemons:Option<String>,
  pub battled_trainers:Option<String>,
  pub find_seed_min_diff_day_seed:u32,
  pub find_seed_max_diff_day_seed:u32,
  pub find_seed_stop_on_match:bool,

  // testing_only
  pub testing_search_nearby_diff_day_adv:Option<u32>,
  pub testing_search_nearby_same_day_adv:Option<u32>,
  pub testing_search_nearby_ignore_rating:bool,

}


impl Options {
  pub fn default() -> Options {
    Options::from_json(&Options::default_json_str())
  }
  pub fn from_json(json:&str) -> Options {
    let res = serde_json::from_str(&json);
    match res {
      Ok(opts) => { opts },
      Err(err) => {
        std::fs::write("from_json.txt", json);
        panic!("{}", err);
      }
    }
  }

  pub fn default_json_str() -> String {
    let mut opts_str = String::from(r#"{
      "errors":[],
      "same_day_seed":null,
      "diff_day_seed":null,
      "exec_obj":0,
      "at_least_one_same_day_adv":true,
      "max_same_day_adv":10,
      "max_diff_day_adv":365240,
      "wins":49,
      "filter_min_rating":0.0,
      "print_rng_frames_info":false,
      "facility":"single",
      "search_update_min_filter":true,
      "player_team_for_multi":null,
      "max_threads":null,
      "player_pokemons_idx_filter":null,
      "find_seed_stop_on_match":false,
      "search_stop_on_perfect_rating":true,
      "find_seed_min_diff_day_seed":0,
      "find_seed_max_diff_day_seed":4294967295,
      "battled_pokemons":null,
      "battled_trainers":null,
      "testing_search_nearby_diff_day_adv":null,
      "testing_search_nearby_same_day_adv":null,
      "testing_search_nearby_ignore_rating":false,
      "player_pokemons":"#);
    opts_str.push_str(PMON_JSON_STR);
    opts_str.push_str("}");
    opts_str
  }

  pub fn initialize(&mut self, exec_mode:u32, args:Vec<&str>){
    self.exec_obj = exec_mode;

    let mut args:Vec<&str> = args.iter().map(|str|{ str.trim() }).collect();
    args.retain(|str| { !str.is_empty() });

    for i in (0..args.len()-1).step_by(2) {
      let key = args[i];
      let val = args[i + 1];
      match key {
        "--wins" => {
          self.wins = val.parse().unwrap_or_else(|_|{
            self.errors.push(format!("Invalid value for argument --wins. Value provided: {}. Expected a number in decimal format. Ex: 0", val));
            0
          });
        },
        "--facility" => {
          self.facility = serde_json::from_str(&format!("\"{}\"", val)).unwrap_or_else(|_|{
            self.errors.push(format!("Invalid value for argument --facility. Value provided: {}. Possible values: single, double", val));
            FacilityEnum::single
          });
        },
        "--print_rng_frames_info" => {
          self.print_rng_frames_info = val.eq("true");
        },

        "--same_day_seed" => {
          if exec_mode != ExecObjective::search_easy &&
             exec_mode != ExecObjective::generate_one {
            println!("Warning: --same_day_seed was provided, but it isn't used for this execution mode.");
          }

          let s = String::from(val);
          self.same_day_seed = Some(s.clone());
          if !validate_str_to_u32(&s) {
            self.errors.push(format!("Invalid value for argument --same_day_seed. Value provided: {}. Expected number in hex format. Ex: 0x123ABC", s));
          }
        },
        "--diff_day_seed" => {
          if exec_mode != ExecObjective::search_easy && exec_mode != ExecObjective::find_seeds {
            println!("Warning: --diff_day_seed was provided, but it isn't used for this execution mode.");
          }

          let s = String::from(val);
          self.diff_day_seed = Some(s.clone());
          if !validate_str_to_u32(&s) {
            self.errors.push(format!("Invalid value for argument --diff_day_seed. Value provided: {}. Expected number in hex format. Ex: 0x123ABC", s));
          }
        },
        "--max_diff_day_change" => {
          if exec_mode != ExecObjective::search_easy {
            println!("Warning: --max_diff_day_change was provided, but it isn't used for this execution mode.");
          }

          const days_2000_to_2099:u32 = (Date::new3(2099,12,31).jd - Date::new3(2000,1,1).jd) as u32; // 36524
          const max_diff_day_change_max_value:u32 = 4294967295 / days_2000_to_2099;
          let wanted_max_diff_day_change = val.parse().unwrap_or_else(|_|{
            self.errors.push(format!("Invalid value for argument --max_diff_day_change. Value provided: {}. Expected number in decimal format. Ex: 0", val));
            0
          });

          self.max_diff_day_adv = if wanted_max_diff_day_change > max_diff_day_change_max_value {
            4294967295
          } else {
            wanted_max_diff_day_change * days_2000_to_2099
          }
        },
        "--max_diff_day_adv" => {
          if exec_mode != ExecObjective::search_easy {
            println!("Warning: --max_diff_day_adv was provided, but it isn't used for this execution mode.");
          }

          self.max_diff_day_adv = val.parse().unwrap_or_else(|_|{
            self.errors.push(format!("Invalid value for argument --max_diff_day_adv. Value provided: {}. Expected number in decimal format. Ex: 0", val));
            0
          });
        },
        "--max_same_day_adv" => {
          if exec_mode != ExecObjective::search_easy {
            println!("Warning: --max_same_day_adv was provided, but it isn't used for this execution mode.");
          }

          self.max_same_day_adv = val.parse().unwrap_or_else(|_|{
            self.errors.push(format!("Invalid value for argument --max_same_day_adv. Value provided: {}. Expected number in decimal format. Ex: 0", val));
            0
          });
        },
        "--at_least_one_same_day_adv" => {
          if exec_mode != ExecObjective::search_easy {
            println!("Warning: --at_least_one_same_day_adv was provided, but it isn't used for this execution mode.");
          }

          self.at_least_one_same_day_adv = val == "true";
        },

        "--find_seed_max_diff_day_seed" => {
          if exec_mode != ExecObjective::find_seeds {
            println!("Warning: --find_seed_max_diff_day_seed was provided, but it isn't used for this execution mode.");
          }

          self.find_seed_max_diff_day_seed = val.parse().unwrap_or_else(|_|{
            self.errors.push(format!("Invalid value for argument --find_seed_max_diff_day_seed. Value provided: {}. Expected number in decimal format. Ex: 0", val));
            0
          });
        },
        "--find_seed_min_diff_day_seed" => {
          if exec_mode != ExecObjective::find_seeds {
            println!("Warning: --find_seed_min_diff_day_seed was provided, but it isn't used for this execution mode.");
          }

          self.find_seed_min_diff_day_seed = val.parse().unwrap_or_else(|_|{
            self.errors.push(format!("Invalid value for argument --find_seed_max_diff_day_seed. Value provided: {}. Expected number in decimal format. Ex: 0", val));
            0
          });
        },
        "--filter_min_rating" => {
          if exec_mode != ExecObjective::search_easy {
            println!("Warning: --filter_min_rating was provided, but it isn't used for this execution mode.");
          }

          self.filter_min_rating = val.parse().unwrap_or_else(|_|{
            self.errors.push(format!("Invalid value for argument --find_seed_max_diff_day_seed. Value provided: {}. Expected number in decimal format. Ex: 14.5", val));
            0.0f32
          });
        },
        "--max_threads" => {
          if exec_mode != ExecObjective::search_easy && exec_mode != ExecObjective::find_seeds {
            println!("Warning: --max_threads was provided, but it isn't used for this execution mode.");
          }

          self.max_threads = Some(val.parse().unwrap_or_else(|_|{
            self.errors.push(format!("Invalid value for argument --max_threads. Value provided: {}. Expected number in decimal format. Ex: 0", val));
            1u64
          }));
        },
        "--find_seed_stop_on_match" => {
          if exec_mode != ExecObjective::find_seeds {
            println!("Warning: --find_seed_stop_on_match was provided, but it isn't used for this execution mode.");
          }

          self.find_seed_stop_on_match = val == "true";
        },
        "--search_stop_on_perfect_rating" => {
          if exec_mode != ExecObjective::search_easy {
            println!("Warning: --search_stop_on_perfect_rating was provided, but it isn't used for this execution mode.");
          }

          self.search_stop_on_perfect_rating = val == "true";
        },
        "--search_update_min_filter" => {
          if exec_mode != ExecObjective::search_easy {
            println!("Warning: --search_update_min_filter was provided, but it isn't used for this execution mode.");
          }
          self.search_update_min_filter = val == "true";
        },
        "--player_pokemons_idx_filter" => {
          if exec_mode == ExecObjective::find_seeds {
            println!("Warning: --player_pokemons_idx_filter was provided, but it isn't used for this execution mode.");
          }

          if exec_mode == ExecObjective::generate_one &&
             val.contains(",") {
            println!("Warning: --player_pokemons_idx_filter with multiple indexes was provided, but only the first index will be used for this execution mode.");
          }

          self.player_pokemons_idx_filter = Some(String::from(val));

          self.player_pokemons_idx_filter.clone().unwrap().split(",").for_each(|id|{
            id.parse::<u32>().unwrap_or_else(|x|{
              self.errors.push(format!("Invalid value for argument --player_pokemons_idx_filter. Value provided: {}. Expected number in decimal format. Ex: 0", x));
              0
            });
          });
        },
        "--battled_pokemons" => {
          if exec_mode != ExecObjective::find_seeds {
            println!("Warning: --battled_pokemons was provided, but it isn't used for this execution mode.");
          }

          self.battled_pokemons = Some(String::from(val));

          self.battled_pokemons_as_vec().iter().for_each(|input_mon_name|{
            let formatted_input_mon_name = format_name(input_mon_name);
            if JTMONS.iter().all(|jtmon|{
              jtmon.species_formatted != formatted_input_mon_name
            }){
              self.errors.push(format!("Error: Invalid Pokemon name provided in  --battled_pokemons. Invalid name: \"{}\"", input_mon_name));
            }
          });
        },
        "--battled_trainers" => {
          if exec_mode != ExecObjective::find_seeds {
            println!("Warning: --battled_trainers was provided, but it isn't used for this execution mode.");
          }

          self.battled_trainers = Some(String::from(val));

          self.battled_trainers_as_vec().iter().for_each(|input_t_name|{
            let formatted_input_t_name = format_name(input_t_name);
            if JTRAINERS.iter().all(|t|{
              t.name_formatted != formatted_input_t_name
            }){
              self.errors.push(format!("Error: Invalid Trainer name provided in --battled_trainers. Invalid name: \"{}\"", input_t_name));
            }
          });
        },
        _ => {
          self.errors.push(format!("Invalid argument: {}", key));
        }
      }
    }

    if self.max_same_day_adv == 0 && self.at_least_one_same_day_adv {
      self.at_least_one_same_day_adv = false;
    }

    for error in self.errors.iter() {
      printerr(&format!("Error: {}", error));
    }

    for (i,pmon) in self.player_pokemons.iter_mut().enumerate(){
      pmon.id = i as u32;
      pmon.init();
    }
  }

  pub fn from_args_str(exec_mode:u32, args_str:&str) -> Options {
    let mut opts = Options::default();
    let mut args_vec:Vec<&str> = args_str.split(' ').collect();
    opts.initialize(exec_mode, args_vec);
    opts
  }


  pub fn from_verb_args_vec(args_vec:Vec<String>) -> Option<Options> {
    let mut args_vec:Vec<&str> = args_vec.iter().map(|s| &**s).collect();
    // bin.exe search_easy --name value
    if args_vec.is_empty() {
      printerr("Error: Not enough arguments provided.");
      return None;
    }

    let verb = args_vec.remove(0);
    let exec_mode = match verb {
      "search_easy" => { ExecObjective::search_easy },
      "generate_one" => { ExecObjective::generate_one },
      "find_seeds" => { ExecObjective::find_seeds },
      _ => {
        printerr(&format!("Error: Invalid first argument {}. Valid values are \"search_easy\", \"generate_one\", \"find_seeds\".", verb));
        return None;
      }
    };

    let mut opts = Options::default();
    opts.initialize(exec_mode, args_vec);
    Some(opts)
  }

  pub fn battled_pokemons_as_vec(&self) -> Vec<String> {
    match self.battled_pokemons.clone() {
      None => { vec![] },
      Some(battled_pokemons) => battled_pokemons.split(",").map(|a|{ String::from(a) }).collect()
    }
  }
  pub fn battled_trainers_as_vec(&self) -> Vec<String> {
    match self.battled_trainers.clone() {
      None => { vec![] },
      Some(battled_trainers) => battled_trainers.split(",").map(|a|{ String::from(a) }).collect()
    }
  }

  pub fn display_help(){
    println!("Read README.md for the list of possible arguments.")
  }
  pub fn player_pokemons_idx_filter(&self) -> Option<Vec<u32>> {
    if let Some(player_pokemons_idx_filter) = &self.player_pokemons_idx_filter {
      Some(player_pokemons_idx_filter.split(",")
                                .map(|id|{ id.parse().unwrap()})
                                .collect())
    } else {
      None
    }
  }
  pub fn fst_player_pokemons_idx_filter(&self) -> Option<usize> {
    let vec = self.player_pokemons_idx_filter();
    if let Some(vec) = vec {
      if vec.is_empty() { None }
      else { Some(vec[0] as usize) }
    } else { None }
  }
}


