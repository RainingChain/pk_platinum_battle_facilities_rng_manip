use std::mem::MaybeUninit;
use std::ops::Div;
use super::{Options,SameDayRNG_rev,State, Strainer,Facility,JTMONS,JTRAINERS,TMONS_COMPATIBILITY_MATRIX,getPossibleTrainersByWins,Filter,log,TeamRNG,SameDayRNG,TeamRNG_rev_multi};

pub struct Generator<const EO:u32,const F:u32, const TC:usize, const PC:usize> {
  pub opts:Options,
  pub filter:Filter<EO>,
  pub possibleTrainers:[Vec<u16>; 14],
  pub isTeammateCompatibleWithPteam:Vec<bool>,
}

static mut frameAdvancedSum:u32 = 0; // for debug

impl<const EO:u32,const F:u32, const TC:usize, const PC:usize> Generator<EO,F,TC,PC> {
  pub fn new(opts:&Options) -> Generator<EO,F,TC,PC> {
    let mut gen = Generator::<EO,F,TC,PC> {
      opts:opts.clone(),
      filter:Filter::new(opts.clone()),
      isTeammateCompatibleWithPteam:Default::default(),
      possibleTrainers: Default::default(),
    };

    // determine all trainer ids
    let trainerCountByBattle = Facility::getTrainerCountByBattle(F);
    let battleCount = Facility::getBattleCount(F);

    for i in 0..battleCount {
      for j in 0..trainerCountByBattle {
        let strainerIdx = (i * trainerCountByBattle + j) as usize;
        let possibleJtrainers = getPossibleTrainersByWins::<F>(
            gen.opts.wins + (i as u32), j as u32);

        gen.possibleTrainers[strainerIdx] = possibleJtrainers.clone();
      }
    }
    gen
  }
  pub fn get_advance_team_seed_pre_streak(&self) -> u32 {
    let round = self.opts.wins / 7;
    round * 24 + 1
  }

  /** same_day_seed is the value after applying date change
   * if no date change, same_day_seed is the same as the value in the .sav
   * if date change, same_day_seed is the value of SameDayRNG(diff_day_seed).
    */
  #[inline(never)] // to be able to cpu profile
  pub fn generate(&mut self, same_day_seed:u32) -> Option<State<TC,PC>>{
    self.filter.reset_for_new_seed();

    let init_team_seed = if self.opts.wins == 0 { SameDayRNG::next32_s(same_day_seed) } else { same_day_seed };
    let mut rng = TeamRNG::new(init_team_seed);
    self.advance(&mut rng, self.get_advance_team_seed_pre_streak(), "pre_streak");
    let post_init_team_seed = rng.get32();
    let mut state = self.generate_with_precomputed_team_seed(post_init_team_seed);

    if state.is_some() {
      let mut state = state.unwrap();
      state.same_day_seed = same_day_seed;
      return Some(state)
    } else {
      None
    }
  }

  /** returns same_day_seed that generates the post_init_team_seed */
  pub fn reverse_compute_team_seed(&mut self, post_init_team_seed:u32) -> u32 {
    let pre_init_init_team_seed = TeamRNG_rev_multi(post_init_team_seed, self.get_advance_team_seed_pre_streak());
    if self.opts.wins == 0 {
      SameDayRNG_rev(pre_init_init_team_seed)
    } else {
      pre_init_init_team_seed
    }
  }

  pub fn generate_with_precomputed_team_seed(&mut self, post_init_team_seed:u32) -> Option<State<TC,PC>>{
    let mut rng = TeamRNG::new(post_init_team_seed);
    let mut state:State<TC,PC> = unsafe { MaybeUninit::uninit().assume_init() };

    let trainerCountByBattle = Facility::getTrainerCountByBattle(F);
    let battleCount = Facility::getBattleCount(F);
    let trainerCount = Facility::getTrainerCount(F);

    self.advance(&mut rng, 0, "selectTrainerStart");

    // determine all trainer ids
    for i in 0..battleCount {
      let mut j = 0;
      while j < trainerCountByBattle {
        let strainerIdx = (i * trainerCountByBattle + j) as usize;

        let possibleJtrainers = unsafe { self.possibleTrainers.get_unchecked(strainerIdx) };
        let len = possibleJtrainers.len();
        let adv = if len > 1 { 1 } else { 0 };
        self.advance(&mut rng, adv, "selectTrainer");

        let jtrainerIdx = rng.get16() as usize % len;
        let jtrainerId = possibleJtrainers[jtrainerIdx];
        if self.isTraceActive() {
          self.log(&String::from(format!(" : Len={}, Idx{:#08x} => Id{}", len, jtrainerIdx, jtrainerId)));
        }

        let already_existed = state.trainers[0..strainerIdx].iter().any(|t|{
          t.trainerId == jtrainerId
        });

        if (already_existed) {
          if self.isTraceActive() { self.log(" : trainer duplicate"); }
          continue;  // We can't have the same trainer twice in the series.
        }
        if !self.filter.doesTrainerRespectFilter::<EO>(jtrainerId, strainerIdx) {
          return None;
        }
        state.trainers[strainerIdx].trainerId = jtrainerId;
        j += 1;
      }
    }
    self.advance(&mut rng, 0, "selectTrainerEnd, selectPokemonStart");

    // determine the pokemons
    for i in 0..battleCount {
      for j in 0..trainerCountByBattle {
        let trainerIdx = (i * trainerCountByBattle + j) as usize;

        let valid = if Facility::isMulti(F) && i % 2 == 1 {
          let (v0,v1) = state.trainers.split_at_mut(trainerIdx);
          self.addPokemonsToTrainer(&mut rng, &mut v1[0], false, Some(&v0[v0.len() - 1]), trainerIdx)
        } else {
          self.addPokemonsToTrainer(&mut rng, &mut state.trainers[trainerIdx], false, None, trainerIdx)
        };

        if !valid
          { return None; }

        if !self.filter.onTrainerSelected::<F, PC>(&state.trainers[trainerIdx], trainerIdx)
          { return None; }
      }

      self.advance(&mut rng, 1, "betweenBattle");
    }
    state.team_seed = post_init_team_seed;
    Some(state)
  }

  fn log(&self,my_str:&str){
    if !self.isTraceActive() { return; }
    print!("{}", my_str);
  }
  fn advance(&self, rng:&mut TeamRNG, adv:u32, my_str:&str){
    if !self.isTraceActive() {
      rng.advance(adv);
    } else {
      if (adv == 0){
        let frameAdvancedSum_tmp = unsafe {frameAdvancedSum};
        print!("\n{} : +{} : {:#08x} -> {:#08x} : {}", frameAdvancedSum_tmp, 0, rng.get32(), rng.get32(), my_str);
        return;
      }

      let mut advToPrint = adv;
      for i in 0..adv {
        let before = rng.get32();
        rng.advance(1);

        let str_to_print = if advToPrint > 0 { my_str.clone() } else { "..." };
        let frameAdvancedSum_tmp = unsafe {frameAdvancedSum};
        print!("\n{} : +{} : {:#08x} -> {:#08x} : {}", frameAdvancedSum_tmp, advToPrint, before, rng.get32(), str_to_print);
        advToPrint = 0;
        unsafe { frameAdvancedSum += 1; }
      }
    }
  }

  #[inline(never)] // to be able to cpu profile
  fn addPokemonsToTrainer(&mut self, rng:&mut TeamRNG,
      trainer:&mut Strainer<PC>, forTeammate:bool, prevTrainerForMulti:Option<&Strainer<PC>>,
      trainerIdx:usize) -> bool {
    let ref possiblePokemons = unsafe { &JTRAINERS[trainer.trainerId as usize].pokemons };

    let pokeCount = PC;

    let mut i:usize = 0;
    while i < pokeCount {
      self.advance(rng, 1, "selectPokemon");

      let len = possiblePokemons.len();
      let idx = rng.get16() as usize % len;
      let pokeIdToAdd = unsafe { *possiblePokemons.get_unchecked(idx) };
      if self.isTraceActive() {
        self.log(&String::from(format!(" : Len={}, Idx{} => Id{}", len, idx, pokeIdToAdd)));
      }

      let is_compatible:bool = {
        let compatible = trainer.pokemons[0..i].iter().all(|mon| {
          self.areCompatiblePokemons(pokeIdToAdd, mon.id)
        });

        if !compatible {
          if self.isTraceActive(){
            let fst_incompatible = trainer.pokemons[0..i].iter().find(|mon| {
              !self.areCompatiblePokemons(pokeIdToAdd, mon.id)
            }).unwrap();
            self.log(&format!(" ({}) is incompatible with already selected stmon ({})", pokeIdToAdd, fst_incompatible.id));
          }
          false
        } else if (forTeammate && !self.isTeammateCompatibleWithPteam[pokeIdToAdd as usize]) {
          if self.isTraceActive(){
            self.log(&format!(" ({}) is incompatible with player team", pokeIdToAdd));
          }
          false
        } else if let Some(prevTrainerForMulti) = prevTrainerForMulti {
          prevTrainerForMulti.pokemons.iter().all(|mon| {
            self.areCompatiblePokemons(pokeIdToAdd, mon.id)
          })
        } else {
          true
        }
      };

      if (!is_compatible) { continue; }

      if !forTeammate && !self.filter.doesStmonRespectFilter::<F>(pokeIdToAdd, trainerIdx * pokeCount + i)
        { return false; }

      trainer.pokemons[i].id = pokeIdToAdd;
      i += 1;
    }


    //TID
    self.advance(rng, 1, "tid lower bits");
    let tid_low = rng.get16();
    self.advance(rng, 1, "tid higher bits");
    let tid_high = rng.get16();
    let tid = (tid_low as u32) | ((tid_high as u32) << 16);

    for i in 0..pokeCount {
      let jtmonid = trainer.pokemons[i].id as usize;

      //PID for nature/ability
      let wanted_nature = unsafe { JTMONS.get_unchecked(jtmonid).nature_id as u32 };

      if self.isTraceActive(){
        self.advance(rng, 0, &format!("natpid start mon {} id{}. wanted_nat={:#0x}", i, jtmonid, wanted_nature));
      }

      while (true) {
        self.advance(rng, 1, "natpid lower bits");
        let pid_low = rng.get16();
        self.advance(rng, 1, "natpid higher bits");
        let pid_high = rng.get16();
        let pid = (pid_low as u32) | ((pid_high as u32) << 16);

        if self.is_shiny(tid, pid)
          { continue; }

        let nature = pid % 25;
        if self.isTraceActive(){
          self.advance(rng, 0, &format!("res nature {}", nature));
        }
        if nature != wanted_nature
          { continue; }

        let ability = pid_low % 2;
        trainer.pokemons[i].ability = ability as u8;

        break;
      }
    }

    return true;
  }

  fn is_shiny(&self, pid:u32, pid2:u32) -> bool {
    let a = (pid & 0xffff0000) >> 16;
    let b = pid & 0xffff;
    let c = (pid2 & 0xffff0000) >> 16;
    let d = pid2 & 0xffff;
    (a^b^c^d) < 8
  }

  #[inline(always)]
  fn isTraceActive(&self) -> bool {
    cfg!(debug_assertions) && self.opts.print_rng_frames_info
  }

  #[inline(always)]
  fn areCompatiblePokemons(&self,pokemon1:u16, pokemon2:u16) -> bool {
    unsafe {
      *TMONS_COMPATIBILITY_MATRIX.get_unchecked((pokemon1 as usize) * 988 + (pokemon2 as usize))
    }
  }

}


