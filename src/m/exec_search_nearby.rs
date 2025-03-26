
use wasm_bindgen::prelude::*;
use std::sync::{Arc, atomic::{Ordering, AtomicBool}};
use std::cmp::min;
use super::{FacilityEnum,log,printerr,Options,Facility,ExecObjective,ProgressDisplayer,DiffDayRNG,SameDayRNG,WasmResult,PkResult,Filter,Generator,str_to_u32,available_parallelism};

struct ExecInfo {
  pub opts:Options,
  pub min_diff_day_adv:u32,
  pub max_diff_day_adv:u32,
  pub incr:usize,
  pub printProgress:bool,
  pub must_stop:Arc<AtomicBool>,
}

fn get_best_result(results_arg:Vec<(Option<PkResult>,u32)>) -> (Option<PkResult>,u32) {
  let results:Vec<PkResult> = results_arg.iter()
    .filter(|res|{ res.0.is_some()})
    .map(|res|{ res.0.clone().unwrap() })
    .collect();

  let sum:u32 = results_arg.iter().map(|res|{ res.1 }).sum();

  (match results.len() {
    0 => { None },
    1 => { Some(results[0].clone()) },
    _ => {
      let el = results.iter().max_by_key(|res|{
        (res.rating * 1000f32) as u64
      });
      Some(el.unwrap().clone())
    }
  }, sum)
}

pub fn exec_find_day_seed_templated<const F:u32, const TC:usize, const PC:usize>(exec_info:&ExecInfo) -> (Option<PkResult>,u32) {

  let mut generator = Generator::<{ExecObjective::search_easy},F,TC,PC>::new(&exec_info.opts);

  let mut result:Option<PkResult> = None;

  let diff_day_count = (exec_info.max_diff_day_adv - exec_info.min_diff_day_adv) as u64 + 1;
  let same_day_count = (exec_info.opts.max_same_day_adv as u64)
                       + (if exec_info.opts.at_least_one_same_day_adv { 0 } else { 1 });

  let progressTodo = diff_day_count * same_day_count / (exec_info.incr as u64);
  let mut progress = ProgressDisplayer::new(progressTodo, exec_info.printProgress);

  let MAX_SCORE = Facility::getPokemonCount(F) as f32;

  let mut res_count:u32 = 0;
  for_each_nearby_seed(&exec_info,|same_day_seed,same_day_seed_adv,diff_day_seed, diff_day_seed_adv| {
    if cfg!(test) {
      if let (Some(only_diff_day_adv),Some(only_same_day_adv)) =
             (exec_info.opts.testing_search_nearby_diff_day_adv,exec_info.opts.testing_search_nearby_same_day_adv) {
        if only_diff_day_adv != diff_day_seed_adv || only_same_day_adv != same_day_seed_adv {
          return false;
        }
      }
    }

    let state = generator.generate(same_day_seed);
    progress.on_progress();

    if progress.progress % 1000 == 0 && exec_info.must_stop.load(Ordering::Relaxed) {
      return true;
    }

    match state {
      None => { return false; },
      Some(state) => {
        let (pmon_idx,rating) = generator.filter.getFinalStateRating_all_pmon::<F,TC,PC>(&state);

        if pmon_idx.is_none() || rating < generator.filter.updatable_filter_min_rating
          { return false; }

        res_count += 1;

        if exec_info.opts.search_update_min_filter {
          generator.filter.updatable_filter_min_rating = rating + 0.01;
          if generator.filter.updatable_filter_min_rating > MAX_SCORE {
            generator.filter.updatable_filter_min_rating = MAX_SCORE;
          }
        }

        let filter = &generator.filter;

        let res_pmon_idx = if exec_info.opts.testing_search_nearby_ignore_rating { None } else { pmon_idx };
        let mut res = PkResult::new(&state, &exec_info.opts, res_pmon_idx, rating);
        res.same_day_seed_adv = Some(same_day_seed_adv);
        res.diff_day_seed_adv = Some(diff_day_seed_adv);
        res.diff_day_seed = Some(diff_day_seed);
        res.clock_date_changes = PkResult::convert_days_adv_to_date(diff_day_seed_adv);

        println!("{}", res.to_minimal_string());
        result = Some(res);

        if exec_info.opts.search_stop_on_perfect_rating && rating >= MAX_SCORE {
          exec_info.must_stop.store(true, Ordering::Relaxed);
          return true;
        }
      }
    }
    return false;
  });
  (result, res_count)
}

fn for_each_nearby_seed<F>(exec_info:&ExecInfo, mut func:F)
    where F: FnMut(u32,u32,u32,u32) -> bool {
  let opts_same_day_seed = str_to_u32(&exec_info.opts.same_day_seed);
  let opts_diff_day_seed = str_to_u32(&exec_info.opts.diff_day_seed);

  /*
  if diff_day_adv == 0
    opts_same_day_seed
    opts_same_day_seed + same_day_adv_1,
    opts_same_day_seed + same_day_adv_2 ...
  else
    (diff_day + diff_day_adv_1)
    (diff_day + diff_day_adv_1) + same_day_adv_1,
    (diff_day + diff_day_adv_1) + same_day_adv_2,

    (diff_day + diff_day_adv_2)
    (diff_day + diff_day_adv_2) + same_day_adv_1,
    (diff_day + diff_day_adv_2) + same_day_adv_2,
    */
  //

  let mut diff_day_seed_if_not_0 = DiffDayRNG::next32_s_multi(opts_diff_day_seed, exec_info.min_diff_day_adv);

  for diff_day_adv in (exec_info.min_diff_day_adv..=exec_info.max_diff_day_adv).step_by(exec_info.incr) {
    let mut same_day_seed_after_diff_adv = if diff_day_adv == 0 {
      opts_same_day_seed
    } else {
      SameDayRNG::next32_s(diff_day_seed_if_not_0)
    };

    if !exec_info.opts.at_least_one_same_day_adv {
      if func(same_day_seed_after_diff_adv, 0, diff_day_seed_if_not_0, diff_day_adv) {
        return
      }
    }
    same_day_seed_after_diff_adv = SameDayRNG::next32_s(same_day_seed_after_diff_adv);

    for same_day_adv in 1..=exec_info.opts.max_same_day_adv {
      if func(same_day_seed_after_diff_adv, same_day_adv, diff_day_seed_if_not_0, diff_day_adv) {
        return
      }
      same_day_seed_after_diff_adv = SameDayRNG::next32_s(same_day_seed_after_diff_adv)
    }

    diff_day_seed_if_not_0 = DiffDayRNG::next32_s_multi(diff_day_seed_if_not_0, exec_info.incr as u32);
  }
}


fn exec_single_thread(exec_info:&ExecInfo) -> (Option<PkResult>,u32) {
    match exec_info.opts.facility {
      FacilityEnum::single => { exec_find_day_seed_templated::<{Facility::single}, 7,3>(exec_info) },
      FacilityEnum::double => { exec_find_day_seed_templated::<{Facility::double}, 7,4>(exec_info) },
    }
}

// the goal is that in case of equal, we want to return the one with the fewest advances
fn split_for_multi_thread(opts:&Options,start:u32, end:u32, max_threads:Option<u64>) -> Vec<ExecInfo> {
  let count = (end - start) as u64 + 1;

  let cpu_threads = available_parallelism();
  let specified_threads_to_use = max_threads.unwrap_or(cpu_threads as u64);
  let threads_to_use = std::cmp::min(specified_threads_to_use as u64, count);

  let must_stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
  if threads_to_use <= 1 {
    return vec![ExecInfo {
      opts:opts.clone(),
      min_diff_day_adv:start,
      max_diff_day_adv:end,
      incr:threads_to_use as usize,
      printProgress:true,
      must_stop:must_stop.clone(),
    }];
  }

  let mut vec:Vec<ExecInfo> = vec![];
  for i in 0..threads_to_use {
    vec.push(ExecInfo {
      opts:opts.clone(),
      min_diff_day_adv:start + i as u32,
      max_diff_day_adv:end,
      incr:threads_to_use as usize,
      printProgress:i == 0,
      must_stop:must_stop.clone(),
    });
  }
  vec
}

pub fn exec_search_nearby_with_count(opts: &Options) -> (Option<PkResult>,u32) {
  crate::m::game_data::init();
  println!("--------------------------------------------------------------------------------------------");
  println!("Searching the number of advances to get the best pokemon series with same day rng seed {} and different day rng seed {}",
           opts.same_day_seed.clone().unwrap(),
           opts.diff_day_seed.clone().unwrap());
  let exec_infos = split_for_multi_thread(opts, 0, opts.max_diff_day_adv, opts.max_threads);
  println!("Thread count: {}", exec_infos.len());
  println!("--------------------------------------------------------------------------------------------");

  let res:(Option<PkResult>,u32) = {
    let incr = exec_infos.len();
    if exec_infos.len() == 1 {
      exec_single_thread(&exec_infos[0])
    } else {
      let handles:Vec<std::thread::JoinHandle<(Option<PkResult>,u32)>> = exec_infos.into_iter().enumerate().map(|(i,exec_info)|{
        let opts = opts.clone();
        std::thread::spawn(move || {
          exec_single_thread(&exec_info)
        })
      }).collect();

      let results:Vec<(Option<PkResult>,u32)> = handles.into_iter().map(|handle|{
        handle.join().unwrap()
      }).collect();

      get_best_result(results)
    }
  };

  println!("");
  println!("--------------------------------------------------------------------------------------------");
  println!("Final result:");
  println!("--------------------------------------------------------------------------------------------");
  match res.0.clone() {
    None => {
      println!("No result.");
    },
    Some(res) => {
      println!("{}", res.to_string());
      println!("{}", res.to_generate_one_command_line_string(opts));
    }
  }
  res
}
pub fn exec_search_nearby(opts: &Options) -> Option<PkResult> {
  exec_search_nearby_with_count(opts).0
}

#[wasm_bindgen]
pub fn exec_search_nearby_wasm(opts_args:&str,min_diff_day_adv:u32,max_diff_day_adv:u32,incr:u32,printProgress:bool) -> String {
  crate::m::game_data::init();

  let opts = Options::from_args_str(ExecObjective::search_easy, opts_args);
  if !opts.errors.is_empty() {
    serde_json::to_string(&WasmResult {
      errors:opts.errors.clone(),
      results:vec![],
    }).unwrap()
  } else {
    let exec_info = ExecInfo {
      opts,
      min_diff_day_adv,
      max_diff_day_adv,
      incr:incr as usize,
      printProgress,
      must_stop:Arc::new(AtomicBool::new(false)),
    };

    let result = exec_single_thread(&exec_info).0;
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

  fn helper_internal(opts_args:&str, testing_search_nearby_diff_day_adv:Option<u32>, testing_search_nearby_same_day_adv:Option<u32>,testing_search_nearby_ignore_rating:bool) -> (Option<PkResult>, u32) {
    let mut opts = Options::default();
    let args_str = String::from(opts_args);
    let mut args:Vec<&str> = args_str.split(" ").collect();
    args.remove(0);
    args.remove(0);
    args.remove(0);
    args.remove(0);
    opts.initialize(ExecObjective::search_easy, args);
    opts.testing_search_nearby_diff_day_adv = testing_search_nearby_diff_day_adv;
    opts.testing_search_nearby_same_day_adv = testing_search_nearby_same_day_adv;
    opts.testing_search_nearby_ignore_rating = testing_search_nearby_ignore_rating;

    if testing_search_nearby_ignore_rating {
      //hack so the result aren't filtered
      for pmon in opts.player_pokemons.iter_mut() {
        for rating in pmon.ratingsByMovePokemonAndAbility.iter_mut() {
          *rating = 1f32;
        }
      }
    }

    exec_search_nearby_with_count(&opts)
  }
  fn helper(opts_args:&str, testing_search_nearby_diff_day_adv:Option<u32>, testing_search_nearby_same_day_adv:Option<u32>,testing_search_nearby_ignore_rating:bool) -> String {
    let res = helper_internal(opts_args, testing_search_nearby_diff_day_adv, testing_search_nearby_same_day_adv, testing_search_nearby_ignore_rating);
    format!("{}", res.0.unwrap().to_test_string())
  }

  #[test]
  fn test_search_nearby_singles_wins7_diff0_same0() {
    assert_eq!(
      helper("cargo run -- search_easy --facility single --same_day_seed 0x4CB8E431 --diff_day_seed 0xcfa835d7 --wins 7 --filter_min_rating 1 --max_diff_day_adv 2 --max_same_day_adv 2 --at_least_one_same_day_adv false",
        Some(0), Some(0), true),

      "0x4cb8e431 (t110=Black Belt Soren:p247=Kabutops 1,a1|p164=Pupitar 1,a1|p213=Hitmonlee 1,a0|), (t117=Socialite Barbara:p225=Tropius 1,a0|p227=Magneton 1,a0|p237=Kingler 1,a1|), (t96=Cameraman Amleth:p136=Illumise 1,a0|p109=Nidorina 1,a1|p110=Nidorino 1,a0|), (t86=Worker Shane:p114=Corsola 1,a0|p90=Rhyhorn 1,a0|p42=Machop 1,a1|), (t103=Cyclist♀ Kaya:p152=Wartortle 1,a0|p199=Kecleon 1,a1|p150=Ivysaur 1,a1|), (t113=Battle Girl Ingrid:p161=Prinplup 1,a0|p196=Vigoroth 1,a0|p167=Gabite 1,a1|), (t137=Scientist Brad:p284=Chimecho 2,a1|p282=Wigglytuff 2,a1|p281=Mothim 2,a0|), "
    );
  }

  #[test]
  fn test_search_nearby_singles_wins7_diff0_same1() {
    assert_eq!(
      helper("cargo run -- search_easy --facility single --same_day_seed 0x4CB8E431 --diff_day_seed 0xcfa835d7 --wins 7 --filter_min_rating 1 --max_diff_day_adv 2 --max_same_day_adv 2 --at_least_one_same_day_adv false",
        Some(0), Some(1), true),

      "0xe6b0a256 (t108=Black Belt Jericho:p157=Combusken 1,a0|p185=Gligar 1,a1|p203=Sandslash 1,a1|), (t86=Worker Shane:p73=Aron 1,a1|p23=Riolu 1,a0|p122=Onix 1,a1|), (t106=Battle Girl Kodi:p220=Zangoose 1,a0|p176=Linoone 1,a1|p243=Relicanth 1,a1|), (t111=Battle Girl Trina:p249=Cloyster 1,a1|p246=Omastar 1,a1|p169=Raticate 1,a1|), (t81=Idol Ada:p123=Lickitung 1,a1|p145=Plusle 1,a0|p108=Dunsparce 1,a0|), (t105=Black Belt Mason:p160=Monferno 1,a0|p214=Hitmonchan 1,a1|p158=Marshtomp 1,a0|), (t125=Psychic♀ Jillian:p281=Mothim 2,a1|p329=Stantler 2,a0|p292=Misdreavus 2,a1|), "
    );
  }


  #[test]
  fn test_search_nearby_singles_wins7_diff1_same0() {
    assert_eq!(
      helper("cargo run -- search_easy --facility single --same_day_seed 0x4CB8E431 --diff_day_seed 0xcfa835d7 --wins 7 --filter_min_rating 1 --max_diff_day_adv 2 --max_same_day_adv 2 --at_least_one_same_day_adv false",
        Some(1), Some(0), true),

      "0x713b6ba5 (t109=Black Belt Harris:p249=Cloyster 1,a1|p244=Electabuzz 1,a0|p171=Furret 1,a1|), (t117=Socialite Barbara:p234=Torkoal 1,a1|p216=Hitmontop 1,a0|p206=Seaking 1,a0|), (t112=Battle Girl Alta:p171=Furret 1,a1|p203=Sandslash 1,a1|p179=Metang 1,a1|), (t86=Worker Shane:p42=Machop 1,a1|p45=Numel 1,a0|p24=Bonsly 1,a0|), (t92=Cowgirl Paisley:p101=Kabuto 1,a0|p117=Mawile 1,a0|p112=Magby 1,a1|), (t89=Rancher Etienne:p131=Munchlax 1,a1|p142=Togetic 1,a0|p112=Magby 1,a1|), (t129=Pokémon Breeder♀ Adriana:p314=Hitmonchan 2,a0|p332=Pidgeot 2,a1|p342=Gorebyss 2,a0|), "
    );
  }

  #[test]
  fn test_search_nearby_singles_wins7_diff50000_same0() {
    //2000-01-01 -> 2099-12-31 (x1), 2000-01-01 -> 2036-11-23
    assert_eq!(
      helper("cargo run -- search_easy --facility single --same_day_seed 0x4CB8E431 --diff_day_seed 0xcfa835d7 --wins 7 --filter_min_rating 1 --max_diff_day_change 2 --max_same_day_adv 2 --at_least_one_same_day_adv false",
        Some(50000), Some(0), true),
      "0x261fb84 (t88=Rancher Pierce:p119=Butterfree 1,a0|p111=Flaaffy 1,a0|p108=Dunsparce 1,a0|), (t101=Cyclist♂ Ignatio:p195=Seadra 1,a1|p178=Shelgon 1,a0|p189=Pelipper 1,a0|), (t83=Idol Basia:p69=Teddiursa 1,a1|p48=Swablu 1,a0|p67=Voltorb 1,a1|), (t105=Black Belt Mason:p190=Lairon 1,a1|p231=Crawdaunt 1,a0|p247=Kabutops 1,a0|), (t111=Battle Girl Trina:p197=Lunatone 1,a1|p154=Quilava 1,a0|p237=Kingler 1,a0|), (t104=Black Belt Clement:p229=Stantler 1,a0|p152=Wartortle 1,a0|p198=Solrock 1,a1|), (t123=Psychic♀ Amelia:p284=Chimecho 2,a1|p335=Grumpig 2,a1|p302=Noctowl 2,a1|), "
    );
  }


  #[test]
  fn test_search_nearby_singles_wins7_diff40000_same3() {
    //2000-01-01 -> 2099-12-31 (x1), 2000-01-01 -> 2009-7-8
    assert_eq!(
      helper("cargo run -- search_easy --facility single --same_day_seed 0x4CB8E431 --diff_day_seed 0xcfa835d7 --wins 7 --filter_min_rating 1 --max_diff_day_change 2 --max_same_day_adv 3 --at_least_one_same_day_adv false",
        Some(40000), Some(3), true),
      "0x70431e03 (t88=Rancher Pierce:p103=Anorith 1,a1|p101=Kabuto 1,a0|p124=Beautifly 1,a0|), (t98=Reporter Marble:p60=Grimer 1,a1|p53=Psyduck 1,a1|p78=Skorupi 1,a0|), (t113=Battle Girl Ingrid:p203=Sandslash 1,a0|p185=Gligar 1,a0|p164=Pupitar 1,a0|), (t91=Cowgirl Doris:p149=Azumarill 1,a0|p102=Lileep 1,a0|p118=Kricketune 1,a1|), (t112=Battle Girl Alta:p205=Chansey 1,a0|p244=Electabuzz 1,a1|p220=Zangoose 1,a0|), (t106=Battle Girl Kodi:p249=Cloyster 1,a1|p152=Wartortle 1,a1|p205=Chansey 1,a1|), (t129=Pokémon Breeder♀ Adriana:p342=Gorebyss 2,a1|p308=Piloswine 2,a1|p320=Zangoose 2,a1|), "
    );
  }

  //cargo test test_search_nearby_perfect_count --release -- --ignored --nocapture
  #[ignore]
  #[test]
  fn test_search_nearby_perfect_count() {
    let min_rating = 19f32;
    for i in 0..54 {
      let cmd = format!("cargo run -- search_easy --facility single --same_day_seed 0 --diff_day_seed 0 --wins 49 --filter_min_rating {} --max_diff_day_change 999999 --max_same_day_adv 0 --search_stop_on_perfect_rating false --search_update_min_filter false --at_least_one_same_day_adv false --player_pokemons_idx_filter {}", min_rating, i);
      let cnt = helper_internal(&cmd, None, None, false).1;
      println!("pmon_idx {} => {} results with rating >= {}", i, cnt, min_rating);
    }
  }

  //cargo test -- --ignored
  #[ignore]
  #[test]
  fn test_search_nearby_debug() {
    assert_eq!(
      helper("cargo run -- search_easy --facility single --same_day_seed 0x4CB8E431 --diff_day_seed 0xcfa835d7 --wins 7 --filter_min_rating 19 --max_diff_day_change 5 --max_same_day_adv 5 --at_least_one_same_day_adv false", None, None, false),
      "0x70431e03 (t88=Rancher Pierce:p103=Anorith 1,a1|p101=Kabuto 1,a1|p124=Beautifly 1,a1|), (t98=Reporter Marble:p60=Grimer 1,a1|p53=Psyduck 1,a1|p78=Skorupi 1,a1|), (t113=Battle Girl Ingrid:p203=Sandslash 1,a0|p185=Gligar 1,a0|p164=Pupitar 1,a0|), (t91=Cowgirl Doris:p149=Azumarill 1,a0|p102=Lileep 1,a0|p118=Kricketune 1,a0|), (t112=Battle Girl Alta:p205=Chansey 1,a0|p244=Electabuzz 1,a0|p220=Zangoose 1,a0|), (t106=Battle Girl Kodi:p249=Cloyster 1,a0|p152=Wartortle 1,a0|p205=Chansey 1,a0|), (t129=Pokémon Breeder♀ Adriana:p342=Gorebyss 2,a0|p308=Piloswine 2,a0|p320=Zangoose 2,a0|), "
    );
  }
}