
use wasm_bindgen::prelude::*;
use std::sync::{Arc, atomic::{Ordering, AtomicBool}};

use super::{FacilityEnum,Facility,ExecObjective,Options,str_to_u32,available_parallelism,Generator,Filter,WasmResult,PkResult,TeamRNG,SameDayRNG_rev,SameDayRNG,DiffDayRNG,DiffDayRNG_rev,ProgressDisplayer};

struct ExecInfo {
  pub opts:Options,
  pub min_diff_day_adv:u32,
  pub max_diff_day_adv:u32,
  pub incr:usize,
  pub printProgress:bool,
  pub must_stop:Arc<AtomicBool>,
}

fn for_each_seed<const F:u32, const TC:usize, const PC:usize,FF>(exec_info:&ExecInfo, diff_day_seed:u32, mut func:FF)
  where FF: FnMut(u32,u32) -> bool {

  let mut diff_day_seed = str_to_u32(&exec_info.opts.diff_day_seed);

  diff_day_seed = DiffDayRNG::next32_s_multi(diff_day_seed, exec_info.min_diff_day_adv);
  for adv in (exec_info.min_diff_day_adv..=exec_info.max_diff_day_adv).step_by(exec_info.incr) {
    let mut same_day_seed = SameDayRNG::next32_s(diff_day_seed);

    if func(same_day_seed, diff_day_seed) {
      return;
    }
    diff_day_seed = DiffDayRNG::next32_s_multi(diff_day_seed, exec_info.incr as u32);
  }
}

pub fn exec_find_day_seed_templated<const F:u32, const TC:usize, const PC:usize>(exec_info:&ExecInfo) -> Vec<PkResult> {
  let mut generator = Generator::<{ExecObjective::find_seeds},F,TC,PC>::new(&exec_info.opts);

  let mut res:Vec<PkResult> = vec![];

  let progressTodo = ((exec_info.max_diff_day_adv - exec_info.min_diff_day_adv) as u64 + 1) / exec_info.incr as u64;
  let mut progress = ProgressDisplayer::new(progressTodo, exec_info.printProgress);

  match exec_info.opts.diff_day_seed {
    None => {
      //optimization: find good team_seed, then reverse to get same_day_seed/diff_day_seed
      for team_seed in (exec_info.min_diff_day_adv..=exec_info.max_diff_day_adv).step_by(exec_info.incr) {
        let state = generator.generate_with_precomputed_team_seed(team_seed);

        if let Some(state) = state {
          let mut result = PkResult::new(&state, &exec_info.opts, None, 0.0);

          result.same_day_seed = generator.reverse_compute_team_seed(team_seed);
          result.diff_day_seed = Some(SameDayRNG_rev(result.same_day_seed));

          res.push(result);
          if exec_info.opts.find_seed_stop_on_match {
            exec_info.must_stop.store(true, Ordering::Relaxed);
            break;
          }
        }

        if exec_info.printProgress {
          progress.on_progress();
        }

        if progress.progress % 1000 == 0 && exec_info.must_stop.load(Ordering::Relaxed) {
          break;
        }
      }
    },
    Some(_) => {
      for_each_seed::<F,TC,PC,_>(exec_info, str_to_u32(&exec_info.opts.diff_day_seed), |same_day_seed,diff_day_seed|{
        let state = generator.generate(same_day_seed);

        if let Some(state) = state {
          let mut result = PkResult::new(&state, &exec_info.opts, None, 0.0);
          result.diff_day_seed = Some(diff_day_seed);

          res.push(result);
          if exec_info.opts.find_seed_stop_on_match {
            exec_info.must_stop.store(true, Ordering::Relaxed);
            return true;
          }
        }

        if exec_info.printProgress {
          progress.on_progress();
        }

        if progress.progress % 1000 == 0 && exec_info.must_stop.load(Ordering::Relaxed) {
          return true;
        }
        return false;
      });
    }
  }

  res
}

fn exec_single_thread(exec_info:&ExecInfo) -> Vec<PkResult> {
  match exec_info.opts.facility {
    FacilityEnum::single => { exec_find_day_seed_templated::<{Facility::single}, 7,3>(exec_info) },
    FacilityEnum::double => { exec_find_day_seed_templated::<{Facility::double}, 7,4>(exec_info) },
  }
}

#[wasm_bindgen]
pub fn exec_find_day_seed_wasm(opts_args:&str,min_diff_day_adv:u32,max_diff_day_adv:u32,incr:u32,printProgress:bool) -> String {
  crate::m::game_data::init();

  let opts = Options::from_args_str(ExecObjective::find_seeds, opts_args);

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

    let results = exec_single_thread(&exec_info);
    serde_json::to_string(&WasmResult {
      errors:vec![],
      results,
    }).unwrap()
  }

}

pub fn split_for_multi_thread(opts:&Options,start:u32, end:u32, max_threads:Option<u64>) -> Vec<ExecInfo> {
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

  let seconds_per_split = count / threads_to_use;
  let mut start_to_use = start;
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


pub fn exec_find_day_seed(opts: &Options) -> Vec<PkResult> {
  crate::m::game_data::init();
  println!("--------------------------------------------------------------------------------------------");
  println!("Finding --same_day_seed and --diff_day_seed matching the battled trainers \"{}\" and battled Pokemons \"{}\"",
            opts.battled_trainers.clone().unwrap_or(String::from("")),
            opts.battled_pokemons.clone().unwrap_or(String::from("")));
  let exec_infos = split_for_multi_thread(opts, opts.find_seed_min_diff_day_seed, opts.find_seed_max_diff_day_seed, opts.max_threads);
  println!("Thread count: {}", exec_infos.len());
  println!("--------------------------------------------------------------------------------------------");


  let results:Vec<PkResult> = {
    if exec_infos.len() == 1 {
      exec_single_thread(&exec_infos[0])
    } else {
      let handles:Vec<std::thread::JoinHandle<Vec<PkResult>>> = exec_infos.into_iter().map(|exec_info|{
        let opts = opts.clone();
        std::thread::spawn(move || {
          exec_single_thread(&exec_info)
        })
      }).collect();

      let results:Vec<Vec<PkResult>> = handles.into_iter().map(|handle|{
        handle.join().unwrap()
      }).collect();

      results.into_iter().flatten().collect()
    }
  };

  println!("");
  println!("--------------------------------------------------------------------------------------------");
  println!("Final result:");
  println!("--------------------------------------------------------------------------------------------");
  if results.is_empty() {
    println!("No result.");
  } else {
    for res in results.iter() {
      if opts.wins == 0 {
        println!("Seeds when the teams were generated: --same_day_seed {:#08x}", res.same_day_seed);
      }
      println!("Seeds after generating the teams (new current values): --same_day_seed {:#08x} --diff_day_seed {:#08x}", SameDayRNG::next32_s(res.same_day_seed), res.diff_day_seed.unwrap());
    }
  }
  results
}


#[cfg(test)]
mod tests {
  use serde::Serialize;

  use super::*;

  fn helper(opts_args:&str) -> String {
    let mut opts = Options::default();
    let args_str = String::from(opts_args);
    let mut args:Vec<&str> = args_str.split(" ").collect();
    for i in 0..5 {
      args.remove(0);
    }
    opts.initialize(ExecObjective::find_seeds, args);
    let results = exec_find_day_seed(&opts);
    let mut results:Vec<(u32,u32)> = results.iter().map(|r|{ (r.same_day_seed, r.diff_day_seed.unwrap()) }).collect();
    results.sort();
    format!("{:?}", results)
  }

  #[test]
  fn test_find_day_seed_double_0wins_single_res() {
    assert_eq!(
      helper("cargo run --release -- find_seeds --facility double --wins 0 --battled_trainers Vincent,Tia,Shane,Lucy,Melanie,Amleth,Kaylene --battled_pokemons Baltoy,Vulpix,Gible,Cyndaquil,Castform,Remoraid,Mudkip,Lombre,Larvitar,Shieldon,Croagunk,Onix,Loudred,Clefairy,Pidgeotto,Turtwig,Munchlax,Yanma,Ledian,Wailmer,Illumise,Murkrow,Aipom,Chatot,Cherrim,Kadabra,Marshtomp,Wormadam"),
      "[(1829313114, 2614015461)]"
    ); // 0x6d091a5a  0x9bceb5e5
  }

  #[test]
  fn test_find_day_seed_double_0wins_many_res() {
    assert_eq!(
      helper("cargo run --release -- find_seeds --facility double --wins 0 --battled_pokemons Baltoy,Vulpix,Gible,Cyndaquil,Castform,Remoraid"),
      "[(322863299, 1519169178), (1829313114, 2614015461), (2685904122, 1626897413), (3034899431, 3764346094), (3652209200, 707102723)]"
    );
  }
}