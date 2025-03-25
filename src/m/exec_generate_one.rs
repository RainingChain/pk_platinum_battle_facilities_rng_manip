
use wasm_bindgen::prelude::*;

use super::{FacilityEnum,Facility,ExecObjective,Options,str_to_u32,Generator,Filter,WasmResult,PkResult};

pub fn exec_generate_one_templated<const F:u32, const TC:usize, const PC:usize>(opts:&Options) -> Option<PkResult> {
  let mut generator = Generator::<{ExecObjective::generate_one},F,TC,PC>::new(&opts);
  let same_day_seed = str_to_u32(&opts.same_day_seed);

  let mut result:Option<PkResult> = None;

  let same_day_seed = str_to_u32(&opts.same_day_seed);
  let state = generator.generate(same_day_seed);
  match state {
    None => { None },
    Some(state) => {
      match opts.fst_player_pokemons_idx_filter() {
        None => { Some(PkResult::new(&state, opts, None, 0f32)) },
        Some(pmon_idx) => {
          let trainers_rating = Filter::<{ExecObjective::search_easy}>::evalTrainersRating(&opts.player_pokemons[pmon_idx as usize], &state);
          Some(PkResult::new(&state, opts, Some(pmon_idx), trainers_rating))
        }
      }
    }
  }
}

pub fn exec_generate_one(opts: Options) -> Option<PkResult> {
  crate::m::game_data::init();
  println!("--------------------------------------------------------------------------------------------");
  println!("Searching a single result from input same day rng seed {}", opts.same_day_seed.clone().unwrap());
  println!("--------------------------------------------------------------------------------------------");

  let res = {
    match opts.facility {
      FacilityEnum::single => { exec_generate_one_templated::<{Facility::single}, 7,3>(&opts) },
      FacilityEnum::double => { exec_generate_one_templated::<{Facility::double}, 7,4>(&opts) },
    }
  };

  println!("");
  println!("--------------------------------------------------------------------------------------------");
  println!("Final result:");
  println!("--------------------------------------------------------------------------------------------");
  match res {
    None => {
      println!("No result.");
      None
    },
    Some(res) => {
      println!("{}", res.to_string());
      Some(res)
    }
  }
}

#[wasm_bindgen]
pub fn exec_generate_one_wasm(opts_args:&str) -> String {
  crate::m::game_data::init();

  let opts = Options::from_args_str(ExecObjective::generate_one, opts_args);
  if !opts.errors.is_empty() {
    serde_json::to_string(&WasmResult {
      errors:opts.errors.clone(),
      results:vec![],
    }).unwrap()
  } else {
    let result = exec_generate_one(opts);
    serde_json::to_string(&WasmResult {
      errors:vec![],
      results:if let Some(result) = result {
        vec![result]
      } else { vec![] }
    }).unwrap()
  }
}

#[cfg(test)]
mod tests {
  use serde::Serialize;

  use super::*;

  fn helper(opts_args:&str) -> String {
    let mut opts = Options::default();
    let args_str = String::from(opts_args);
    let mut args:Vec<&str> = args_str.split(" ").collect();
    args.remove(0);
    args.remove(0);
    args.remove(0);
    args.remove(0);
    opts.initialize(ExecObjective::generate_one, args);
    exec_generate_one(opts).unwrap().to_test_string()
  }

  #[test]
  fn test_generate_one_single_0wins() {
    assert_eq!(
      helper("cargo run -- generate_one --same_day_seed 0x15BC6A14 --facility single --wins 0 --print_rng_frames_info false"),
      "0x15bc6a14 (t95=Cameraman Xenon:p84=Staryu 1,a1|p63=Eevee 1,a1|p64=Shellos 1,a1|), (t25=Tuber♂ Hank:p49=Glameow 1,a0|p75=Luvdisc 1,a1|p63=Eevee 1,a0|), (t39=Pokéfan♀ Alexia:p5=Totodile 1,a0|p44=Carvanha 1,a0|p8=Mudkip 1,a1|), (t29=Tuber♀ Tia:p68=Chinchou 1,a1|p57=Clefairy 1,a0|p121=Poliwhirl 1,a1|), (t85=Jogger Dave:p144=Wobbuffet 1,a1|p142=Togetic 1,a0|p102=Lileep 1,a1|), (t73=Guitarist Curtis:p26=Duskull 1,a1|p22=Chingling 1,a0|p58=Magnemite 1,a1|), (t105=Black Belt Mason:p171=Furret 1,a0|p154=Quilava 1,a0|p190=Lairon 1,a1|), "
    );
  }
  #[test]
  fn test_generate_one_single_7wins() {
    assert_eq!(
      helper("cargo run -- generate_one --same_day_seed 0x6d091a5a --facility single --wins 7 --print_rng_frames_info false"),
      "0x6d091a5a (t115=Veteran Eric:p201=Fearow 1,a0|p210=Carnivine 1,a1|p245=Magmar 1,a0|), (t109=Black Belt Harris:p246=Omastar 1,a0|p245=Magmar 1,a1|p166=Bibarel 1,a0|), (t91=Cowgirl Doris:p138=Wailmer 1,a0|p121=Poliwhirl 1,a1|p146=Minun 1,a1|), (t118=Socialite Isabel:p221=Seviper 1,a1|p218=Dusclops 1,a0|p227=Magneton 1,a1|), (t93=Cowgirl Leslie:p147=Pachirisu 1,a0|p107=Spinda 1,a0|p103=Anorith 1,a1|), (t85=Jogger Dave:p138=Wailmer 1,a1|p119=Butterfree 1,a0|p122=Onix 1,a0|), (t127=Pokémon Breeder♂ Derrell:p313=Hitmonlee 2,a1|p327=Magneton 2,a0|p338=Cacturne 2,a0|), "
    );
  }
  #[test]
  fn test_generate_one_single_14wins() {
    assert_eq!(
      helper("cargo run -- generate_one --same_day_seed 0x6d091a5a --facility single --wins 14 --print_rng_frames_info false"),
      "0x6d091a5a (t113=Battle Girl Ingrid:p212=Primeape 1,a0|p216=Hitmontop 1,a1|p154=Quilava 1,a0|), (t120=Psychic♂ Alpha:p344=Electabuzz 2,a1|p282=Wigglytuff 2,a0|p318=Dusclops 2,a1|), (t105=Black Belt Mason:p194=Persian 1,a1|p245=Magmar 1,a1|p172=Luxio 1,a0|), (t117=Socialite Barbara:p201=Fearow 1,a0|p211=Golbat 1,a1|p227=Magneton 1,a0|), (t127=Pokémon Breeder♂ Derrell:p335=Grumpig 2,a1|p312=Primeape 2,a1|p328=Mantine 2,a0|), (t114=Veteran Beck:p218=Dusclops 1,a0|p210=Carnivine 1,a0|p242=Gorebyss 1,a1|), (t300=Tower Tycoon Palmer:p952=Dragonite 5,a1|p950=Rhyperior 1,a0|p951=Milotic 5,a1|), "
    );
  }


  #[test]
  fn test_generate_one_double_0wins() {
    assert_eq!(
      helper("cargo run -- generate_one --same_day_seed 0x6d091a5a --facility double --wins 0 --print_rng_frames_info false"),
      "0x6d091a5a (t52=Ninja Boy Vincent:p35=Baltoy 1,a0|p27=Vulpix 1,a0|p39=Gible 1,a1|p4=Cyndaquil 1,a0|), (t29=Tuber♀ Tia:p134=Castform 1,a0|p33=Remoraid 1,a1|p8=Mudkip 1,a0|p86=Lombre 1,a0|), (t86=Worker Shane:p34=Larvitar 1,a0|p97=Shieldon 1,a1|p40=Croagunk 1,a1|p122=Onix 1,a0|), (t69=Aroma Lady Lucy:p106=Loudred 1,a0|p57=Clefairy 1,a1|p94=Pidgeotto 1,a0|p9=Turtwig 1,a0|), (t22=Picnicker Melanie:p131=Munchlax 1,a1|p130=Yanma 1,a1|p128=Ledian 1,a0|p138=Wailmer 1,a0|), (t96=Cameraman Amleth:p136=Illumise 1,a1|p143=Murkrow 1,a1|p104=Aipom 1,a0|p140=Chatot 1,a0|), (t102=Cyclist♀ Kaylene:p173=Cherrim 1,a1|p177=Kadabra 1,a0|p158=Marshtomp 1,a0|p180=Wormadam 1,a1|), "
    );
  }

  #[test]
  fn test_generate_one_double_7wins() {
    assert_eq!(
      helper("cargo run -- generate_one --same_day_seed 0xb6d64383 --facility double --wins 7 --print_rng_frames_info false"),
      "0xb6d64383 (t108=Black Belt Jericho:p154=Quilava 1,a1|p234=Torkoal 1,a0|p243=Relicanth 1,a0|p161=Prinplup 1,a0|), (t100=Cyclist♂ Gaspar:p174=Dragonair 1,a0|p157=Combusken 1,a1|p165=Sealeo 1,a1|p169=Raticate 1,a1|), (t103=Cyclist♀ Kaya:p187=Sneasel 1,a0|p191=Tangela 1,a0|p190=Lairon 1,a0|p199=Kecleon 1,a1|), (t93=Cowgirl Leslie:p145=Plusle 1,a1|p108=Dunsparce 1,a0|p127=Graveler 1,a1|p106=Loudred 1,a0|), (t99=Reporter Gingham:p114=Corsola 1,a1|p130=Yanma 1,a0|p129=Ariados 1,a0|p123=Lickitung 1,a1|), (t88=Rancher Pierce:p145=Plusle 1,a1|p126=Weepinbell 1,a0|p149=Azumarill 1,a0|p141=Haunter 1,a0|), (t126=Pokémon Breeder♂ Howard:p302=Noctowl 2,a0|p328=Mantine 2,a1|p313=Hitmonlee 2,a1|p323=Sharpedo 2,a0|), "
    );
  }
}