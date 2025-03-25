use super::{JTRAINERS,JTMONS,Pmon};

#[derive(Debug, Copy, Clone)]
pub struct Stmon {
  pub id:u16,
  pub ability:u8,
}

#[derive(Debug)]
pub struct Strainer<const PC:usize> {
  pub trainerId:u16,
  pub pokemons:[Stmon;PC],
}

impl<const PC:usize> Strainer<PC> {
}

pub struct State<const TC:usize,const PC:usize>{
  pub trainers:[Strainer<PC>;TC],
  pub multi_teammate:Strainer<PC>,
  pub same_day_seed:u32,
  pub team_seed:u32,
}


