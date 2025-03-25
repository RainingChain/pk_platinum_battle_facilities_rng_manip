use serde::{Serialize, Deserialize};

use super::{Jtrainer, Jtmon, Options,Pmon, Date, SameDayRNG_rev, State,Stmon, Filter, JTMONS,JTRAINERS,ExecObjective,Strainer};

#[derive(Serialize, Deserialize)]
pub struct WasmResult {
  pub results:Vec<PkResult>,
  pub errors:Vec<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PkResult {
  pub team_seed:u32,
  pub same_day_seed:u32,
  pub same_day_seed_adv:Option<u32>,
  pub diff_day_seed:Option<u32>,
  pub diff_day_seed_adv:Option<u32>,
  pub jtrainers:Vec<Rtrainer>,
  pub rating:f32,
  pub pmon_idx:Option<usize>,
  pub pmon_desc:String,
  pub clock_date_changes:Vec<String>, //only for wasm
  pub facility:u32,
  pub wins:u32,
}

#[derive(Debug,Clone,Serialize, Deserialize)]
pub struct Rtrainer {
  pub trainerId:u16,
  pub rtmons:Vec<Rtmon>,
  pub move_name:Option<String>,
  pub rating:Option<f32>,
}

#[derive(Debug,Clone,Serialize, Deserialize)]
pub struct Rtmon {
  pub id:u16,
  pub ability:u8,
  pub rating:Option<f32>,
}


impl Rtrainer {
  pub fn to_full_desc_string(&self) -> String {
    //trainer
    let mut str = format!("{}", JTRAINERS[self.trainerId as usize].name);
    if let Some(rating) = self.rating {
      str.push_str(&format!(" (Rating: {}", rating));

      if let Some(move_name) = self.move_name.as_ref() {
        str.push_str(&format!(", Player Choice Move: {}", move_name));
      }
      str.push_str(")")
    }
    str.push_str("\n");

    //rtmons
    for rtmon in self.rtmons.iter() {
      let ab = &JTMONS[rtmon.id as usize].get_ability_name(rtmon.ability as usize);
      str.push_str(&format!("  {} w/ {}", JTMONS[rtmon.id as usize].display_name, ab));

      if let Some(rating) = rtmon.rating {
        str.push_str(&format!(" (Rating: {})", rating));
      }
      str.push_str("\n");
    }
    str
  }

  pub fn to_string(&self, for_test:bool) -> String {
    if for_test {
      self.to_test_string()
    } else {
      self.to_full_desc_string()
    }
  }
  pub fn to_test_string(&self) -> String {
    let mut str = format!("(t{}={}:", self.trainerId, JTRAINERS[self.trainerId as usize].name);

    for rtmon in self.rtmons.iter() {
      str.push_str(&format!("p{}={},a{}", rtmon.id, JTMONS[rtmon.id as usize].display_name, rtmon.ability));

      if let Some(rating) = rtmon.rating {
        str.push_str(&format!(",r{}", rating));
      }
      str += "|";
    }
    str += "), ";
    str
  }
}

impl PkResult {
  pub fn to_test_string(&self) -> String {
    let rating_txt = if let Some(pmon_idx) = self.pmon_idx {
      format!(", Rating: {} (pmon_idx: {}), ", self.rating, pmon_idx)
    } else {
      String::from(" ")
    };
    format!("{:#08x}{}{}", self.same_day_seed, rating_txt, self.rtrainers_to_string(true))
  }
  pub fn rtrainers_to_string(&self, for_test:bool) -> String {
    let mut str = String::from("");
    for rtrainer in self.jtrainers.iter() {
      str.push_str(&rtrainer.to_string(for_test));
    }
    str
  }

  pub fn convert_days_adv_to_date(day_count:u32) -> Vec<String> {
    let mut day_2000 = Date::new3(2000,1,1);
    let days_2000_to_2099 = (Date::new3(2099,12,31).jd - day_2000.jd) as u32;

    let full_cycle_count = day_count / days_2000_to_2099;
    let reminder = day_count % days_2000_to_2099;
    let mut str:Vec<String> = vec![];
    if full_cycle_count > 0 {
      str.push(format!("2000-01-01 -> 2099-12-31 (x{}), ", full_cycle_count));
    }
    if reminder > 0 {
      day_2000.addDays(reminder as i32);
      str.push(format!("2000-01-01 -> {}", day_2000.toInputString()));
    }
    str
  }
  pub fn to_minimal_string(&self) -> String {
    let mut str = String::from("");

    if let Some(pmon_idx) = self.pmon_idx {
      str.push_str(&format!("Rating: {:.1} #{},", self.rating, pmon_idx));
    }
    str.push_str(&format!("  --same_day_seed {:#08x} diff_day_seed_adv={:?}, same_day_seed_adv={:?}", self.same_day_seed, self.diff_day_seed_adv, self.same_day_seed_adv));
    str
  }
  pub fn to_string(&self) -> String {
    let mut str = String::from("");

    if let Some(pmon_idx) = self.pmon_idx {
      str.push_str(&format!("Rating: {:.1} with {} (#{})\n", self.rating, self.pmon_desc, pmon_idx));
    }
    if let (Some(diff_day_seed_adv),
            Some(diff_day_seed),
            Some(same_day_seed_adv)) = (self.diff_day_seed_adv, self.diff_day_seed, self.same_day_seed_adv) {
      str.push_str(&format!("  Different day RNG advance: {} days, {}\n", diff_day_seed_adv, PkResult::convert_days_adv_to_date(diff_day_seed_adv).join(",")));
      str.push_str(&format!("     Different day seed after advances: {:#08x}\n", diff_day_seed));
      str.push_str(&format!("  Same day RNG advance (start and abandon streak): {}\n", same_day_seed_adv));
    }

    str.push_str(&format!("  Same day Seed (after day update): {:#08x}, Team Seed: {:#08x}\n",
      self.same_day_seed, self.team_seed));

    str.push_str(&format!("{}", self.rtrainers_to_string(false)));
    str
  }
  pub fn to_generate_one_command_line_string(&self, opts:&Options) -> String {
    let mut str = format!("generate_one --facility {} --wins {} --same_day_seed {:08x}",
                          opts.facility.to_str(), opts.wins, self.same_day_seed);
    if let Some(pmon_idx) = self.pmon_idx {
      str.push_str(&format!(" --player_pokemons_idx_filter {}", pmon_idx));
    }
    str
  }
  pub fn to_search_nearby_command_line_string(&self, opts:&Options) -> String {
    let mut str = format!("search_easy --facility {} --wins {} --same_day_seed {:08x} --max_diff_day_change 0 --max_same_day_adv 0",
                          opts.facility.to_str(), opts.wins, self.same_day_seed);
    if let Some(pmon_idx) = self.pmon_idx {
      str.push_str(&format!(" --player_pokemons_idx_filter {}", pmon_idx));
    }
    str
  }
  pub fn new<const TC:usize, const PC:usize>(state:&State<TC,PC>, opts:&Options, pmon_idx: Option<usize>, trainers_rating:f32) -> PkResult {
    let mut jtrainers:Vec<Rtrainer> = vec![];
    for t in state.trainers.iter() {

      let (trainer_rating,move_idx) = {
        match pmon_idx {
          None => { (0f32,0) },
          Some(pmon_idx) => { Filter::<{ExecObjective::search_easy}>::evalTrainerRating::<PC>(&opts.player_pokemons[pmon_idx], t) }
        }
      };

      jtrainers.push(Rtrainer {
        move_name:if let Some(pmon_idx) = pmon_idx {
          if opts.player_pokemons[pmon_idx].ratingsLen > 1 {
            Some(opts.player_pokemons[pmon_idx].moves[move_idx as usize].clone())
          } else {
            None
          }
        } else { None },
        rating:Some(trainer_rating),
        trainerId:t.trainerId,
        rtmons:t.pokemons.iter().map(|stmon|{
          Rtmon {
            id:stmon.id,
            ability:stmon.ability,
            rating:if let Some(pmon_idx) = pmon_idx {
              Some(opts.player_pokemons[pmon_idx].getPokemonRating(stmon, move_idx as usize))
            } else { None }
          }
        }).collect(),
      })
    };

    let res = PkResult {
      team_seed:state.team_seed,
      same_day_seed:state.same_day_seed,
      same_day_seed_adv:None,
      diff_day_seed:None,
      diff_day_seed_adv:None,
      pmon_idx,
      pmon_desc:if let Some(pmon_idx) = pmon_idx {
        opts.player_pokemons[pmon_idx].description.clone()
      } else { String::from("") },
      rating:trainers_rating,
      jtrainers,
      clock_date_changes:vec![],
      facility:opts.facility.tmp(),
      wins:opts.wins,
    };

    res
  }
}