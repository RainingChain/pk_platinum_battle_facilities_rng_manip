use super::{Stmon, State, Strainer,Pmon,Options,Facility,JTMONS,JTRAINERS,format_name,ExecObjective};

pub struct Filter<const EO:u32> {
  pub opts:Options,
  /** the first mon of each trainer must have perfect rating */
  pub search_nearby_jtmonsRespectingFilter_1st_mon:Vec<bool>,
  /** the 2nd+ mon of each trainer must have a rating which permit respecting filter_min_rating */
  pub search_nearby_jtmonsRespectingFilter_2nd_mon:Vec<bool>,
  pub find_day_seed_jtmonsRespectingFilterByIdx:Vec<Vec<bool>>,
  pub max_rating_by_pmon:Vec<(u32,f32)>,
  pub initial_max_rating_by_pmon:Vec<(u32,f32)>,
  pub updatable_filter_min_rating:f32,
  pub pmon_idx_to_delete:Vec<bool>,
  pub find_day_seed_trainer:[Option<u16>;7],

  max_possible_rating:f32,
}

impl<const EO:u32> Filter<EO> {
  pub fn new(opts:Options) -> Filter<EO> {
    let mut filter = Filter {
      search_nearby_jtmonsRespectingFilter_1st_mon:Filter::<EO>::init_get_jtmonsRespectingFilter_1st_mon(&opts),
      search_nearby_jtmonsRespectingFilter_2nd_mon:Filter::<EO>::init_get_jtmonsRespectingFilter_2nd_mon(&opts),
      find_day_seed_jtmonsRespectingFilterByIdx:Filter::<EO>::init_get_jtmonsRespectingFilterByIdx(&opts),
      max_rating_by_pmon:vec![],
      initial_max_rating_by_pmon:vec![],
      updatable_filter_min_rating:opts.filter_min_rating,
      pmon_idx_to_delete:vec![false; opts.player_pokemons.len()],
      max_possible_rating:Facility::getPokemonCount(opts.facility.tmp()) as f32,
      find_day_seed_trainer:Filter::<EO>::init_get_find_day_seed_trainer(&opts),
      opts,
    };

    if EO == ExecObjective::search_easy {
      filter.initial_max_rating_by_pmon.resize(filter.opts.player_pokemons.len(), (0u32,0f32)); //could be faster with MaybeUninit
      for (i,max_rating) in filter.initial_max_rating_by_pmon.iter_mut().enumerate() {
        max_rating.0 = i as u32;
        max_rating.1 = filter.max_possible_rating;
      }
      if let Some(player_pokemons_idx_filter) = filter.opts.player_pokemons_idx_filter() {
        filter.initial_max_rating_by_pmon.retain(|grp|{
          player_pokemons_idx_filter.iter().any(|idx_to_keep|{
            grp.0 == *idx_to_keep
          })
        });
      }
      filter.max_rating_by_pmon = filter.initial_max_rating_by_pmon.clone();
    }
    filter
  }

  pub fn reset_for_new_seed(&mut self){
    if EO == ExecObjective::search_easy {
      unsafe { self.max_rating_by_pmon.set_len(self.initial_max_rating_by_pmon.len()); }
      self.max_rating_by_pmon.copy_from_slice(&self.initial_max_rating_by_pmon);
      //self.pmon_idx_to_delete.fill(false); //should already be done by the user
    }
  }


  fn init_get_find_day_seed_trainer(opts:&Options) -> [Option<u16>;7] {
    let mut ret:[Option<u16>;7] = Default::default();
    for (i,trainer) in opts.battled_trainers_as_vec().iter().enumerate() {
      if i >= ret.len() { break; }

      let formatted_trainer = format_name(trainer);
      if formatted_trainer == format_name("Palmer") {
        ret[i] = Some(if opts.wins == 14 { 300 } else {301});
      } else {
        let t = JTRAINERS.iter().find(|t|{
          t.name_formatted == formatted_trainer
        });
        if let Some(t) = t {
          ret[i] = Some(t.id);
        } else {
          panic!("Invalid name {}", trainer) //should be validated in the options
        }
      }
    }
    ret
  }
  fn init_get_jtmonsRespectingFilter_1st_mon(opts:&Options) -> Vec<bool> {
    if EO != ExecObjective::search_easy {
      JTMONS.iter().map(|jtmon| { true }).collect()
    } else {
      JTMONS.iter().map(|jtmon|{
        (0..2).any(|ability|{
          opts.player_pokemons.iter().any(|pmon|{
            let stmon = Stmon {ability, id:jtmon.id};
            (0..pmon.ratingsByMove.len()).any(|move_idx|{
              pmon.getPokemonRating(&stmon, move_idx) == 1.0f32
            })
          })
        })
      }).collect()
    }
  }

  fn init_get_jtmonsRespectingFilter_2nd_mon(opts:&Options) -> Vec<bool> {
    if EO != ExecObjective::search_easy {
      JTMONS.iter().map(|jtmon| { true }).collect()
    } else {
      let max_score = Facility::getPokemonCount(opts.facility.tmp()) as f32; // ex: 21
      let gap = max_score - opts.filter_min_rating; //ex: 21 - 20.5 => gap=0.5
      //if a mon has a score below min_score_for_mon (0.5), then filter_min_rating won't be respected even if all other mons have a score of 1.
      let min_score_for_mon = 1f32 - gap;

      JTMONS.iter().map(|jtmon|{
        (0..2u8).any(|ability|{
          opts.player_pokemons.iter().any(|pmon|{
            let stmon = Stmon {ability, id:jtmon.id};
            (0..pmon.ratingsByMove.len()).any(|move_idx|{
              pmon.getPokemonRating(&stmon,move_idx) >= min_score_for_mon
            })
          })
        })
      }).collect()
    }
  }

  fn init_get_jtmonsRespectingFilterByIdx(opts:&Options) -> Vec<Vec<bool>> {
    let mon_count = Facility::getPokemonCount(opts.facility.tmp());
    let mut res:Vec<Vec<bool>> = (0..mon_count).map(|idx|{
      JTMONS.iter().map(|_| { true }).collect()
    }).collect();

    if EO != ExecObjective::find_seeds {
      return res;
    }

    let mon_count_by_trainer = Facility::getPokemonCountByTrainer(opts.facility.tmp());
    let battled_mons = opts.battled_pokemons_as_vec();

    for trainer_idx in 0..Facility::getTrainerCount(opts.facility.tmp()) {

      let fst_mon_idx = trainer_idx * mon_count_by_trainer;

      //handle first mon
      {
        if fst_mon_idx >= battled_mons.len() {
          break; // keep all true
        }

        let mon_name = format_name(&battled_mons[fst_mon_idx]);
        res[fst_mon_idx].iter_mut().enumerate().for_each(|(jtmon_idx,val)|{
          *val = JTMONS[jtmon_idx].species_formatted == mon_name;
        });
      }

      //handle 2nd+ mon
      {
        let last_mon_idx = fst_mon_idx + mon_count_by_trainer - 1;
        if last_mon_idx >= battled_mons.len() {
          break; // the full team was not provided, dont consider it
        }

        let r = (fst_mon_idx+1)..=last_mon_idx;

        res[last_mon_idx].iter_mut().enumerate().for_each(|(jtmon_idx,val)|{
          *val = r.clone().any(|mon_idx|{
            let mon_name = format_name(&battled_mons[mon_idx]);
            JTMONS[jtmon_idx].species_formatted == mon_name
          });
        });

        for mon_idx in (fst_mon_idx+1)..last_mon_idx {
          res[mon_idx] = res[last_mon_idx].clone();
        }
      }
    }
    res
  }
  pub fn doesStmonRespectFilter<const F:u32>(&self, monId:u16, jtmon_idx:usize) -> bool {
    match EO {
      ExecObjective::generate_one => {
        true
      },
      ExecObjective::search_easy => {
        let is_fst_trainer_mon = jtmon_idx % Facility::getPokemonCountByTrainer(F) == 0;
        if is_fst_trainer_mon {
          unsafe { *self.search_nearby_jtmonsRespectingFilter_1st_mon.get_unchecked(monId as usize) }
        } else {
          unsafe { *self.search_nearby_jtmonsRespectingFilter_2nd_mon.get_unchecked(monId as usize) }
        }
      },
      ExecObjective::find_seeds => {
        unsafe { *self.find_day_seed_jtmonsRespectingFilterByIdx.get_unchecked(jtmon_idx).get_unchecked(monId as usize) }
      },
      _ => { panic!() }
    }
  }
  pub fn doesTrainerRespectFilter<const F:u32>(&self, t:u16, t_idx:usize) -> bool {
    match EO {
      ExecObjective::generate_one | ExecObjective::search_easy => {
        true
      },
      ExecObjective::find_seeds => {
        unsafe {
          let val = *self.find_day_seed_trainer.get_unchecked(t_idx);
          if let Some(v) = val { v == t } else { true }
        }
      },
      _ => { panic!() }
    }
  }

  // with min_rating=21, about ~50% of exec time is spent in this function
  // possible optimisation: if tmon has bad score for all ratingLen, skip everything
  pub fn evalTrainerRating<const PC:usize>(pmon:&Pmon, trainer:&Strainer<PC>) -> (f32,u8) {
    let mut best_rating:f32 = 0.0;
    let mut best_move_idx:u8 = 0;

    (0..pmon.ratingsLen).for_each(|move_idx|{
      let mut rating:f32 = {
        let r0 = pmon.getPokemonRating(&trainer.pokemons[0], move_idx);
        if r0 != 1.0 { // first mon must have perfect score
          0f32
        } else {
          let mut rating:f32 = 0f32;
          rating += r0;
          for trainerPokemon in trainer.pokemons.iter().skip(1) {
            rating += pmon.getPokemonRating(&trainerPokemon, move_idx);
          }
          rating
        }
      };
      if rating > best_rating {
        best_rating = rating;
        best_move_idx = move_idx as u8;
      }
    });
    (best_rating,best_move_idx)
  }
  pub fn evalTrainersRating<const TC:usize, const PC:usize>(pmon:&Pmon, state:&State<TC,PC>) -> f32 {
    let mut rating = 0f32;
    for trainer in state.trainers.iter() {
      rating += Filter::<EO>::evalTrainerRating::<PC>(pmon, trainer).0;
    }
    rating
  }

  #[inline(never)] // to be able to cpu profile
  pub fn onTrainerSelected<const F:u32, const PC:usize>(&mut self, trainer:&Strainer<PC>, trainerIdx:usize) -> bool {
    if EO != ExecObjective::search_easy {
      return true;
    }

    if Facility::isMulti(F) {   // filter is based on 2 trainers
      return true;
    }

    let mut at_least_one_valid = false;
    let mut at_least_one_deleted = false;

    for (pmon_idx,max_rating) in self.max_rating_by_pmon.iter_mut() {
      let best_rating = PC as f32;

      let (rating,_) = Filter::<EO>::evalTrainerRating::<PC>(&self.opts.player_pokemons[(*pmon_idx) as usize], trainer);

      *max_rating -= (best_rating - rating);

      if *max_rating < self.updatable_filter_min_rating {
        self.pmon_idx_to_delete[(*pmon_idx) as usize] = true;
        at_least_one_deleted = true;
        continue;
      }

      at_least_one_valid = true;
    }

    if at_least_one_deleted {
      self.max_rating_by_pmon.retain(|grp|{
        if self.pmon_idx_to_delete[grp.0 as usize] {
          self.pmon_idx_to_delete[grp.0 as usize] = false;
          false
        } else {
          true
        }
      });
    }

    at_least_one_valid
  }
  /// state must have all trainers set
  pub fn getFinalStateRating_all_pmon<const F:u32,const TC:usize, const PC:usize>(&self, state:&State<TC,PC>) -> (Option<usize>, f32){
    let mut best:(Option<usize>, f32) = (None, 0f32);

    for rating in self.max_rating_by_pmon.iter() {
      if (rating.1 > best.1){
        best.0 = Some(rating.0 as usize);
        best.1 = rating.1;
      }
    }
    best
  }
}
