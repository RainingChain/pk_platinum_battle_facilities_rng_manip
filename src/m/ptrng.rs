
pub struct RNG<const ADD:u32, const MULT: u32> {
  pub seed:u32,
}

impl<const ADD:u32, const MULT: u32> RNG<ADD,MULT> {

  pub fn new(seed:u32) -> RNG<ADD,MULT>
  {
    RNG::<ADD,MULT>{seed}
  }
  pub fn next32_s(seed:u32) -> u32
  {
    seed.wrapping_mul(MULT).wrapping_add(ADD)
  }
  pub fn next32_s_multi(mut seed:u32, count:u32) -> u32
  {
    for i in 0..count {
      seed = seed.wrapping_mul(MULT).wrapping_add(ADD);
    }
    seed
  }
  pub fn next32(&mut self) -> u32
  {
    self.seed = RNG::<ADD,MULT>::next32_s(self.seed);
    self.seed
  }
  pub fn next16(&mut self) -> u16
  {
    (self.next32() / 65535) as u16
  }
  pub fn advance(&mut self, advances:u32){
    for _ in 0..advances
    {
      self.next32();
    }
  }
  pub fn get32(&self) -> u32
  {
    self.seed
  }
  pub fn get16(&self) -> u16
  {
    ((self.seed / 65535) & 0xFFFF) as u16
  }
}

/** used for team (trainer/pokemons) generation */
pub type TeamRNG = RNG<1, 48828125>;

/** used when day changed and when starting new streak.
 * if changing day AND starting new streak, it is called twice */
pub type SameDayRNG = RNG<1, 1566083941>;

/** used for each day increment */
pub type DiffDayRNG = RNG<1, 1812433253>;

pub fn DiffDayRNG_rev(res:u32) -> u32 {
  // fp = y => ((y - 1n) * 2520285293n) % 4294967296n;
  res.wrapping_sub(1).wrapping_mul(2520285293)
}

pub fn SameDayRNG_rev(res:u32) -> u32 {
  // fp = y => ((y - 1n) * 1786162797n) % 4294967296n;
  res.wrapping_sub(1).wrapping_mul(1786162797)
}

pub fn TeamRNG_rev(res:u32) -> u32 {
  // fp = y => ((y - 1n) * 210844021n) % 4294967296n;
  res.wrapping_sub(1).wrapping_mul(210844021)
}

pub fn TeamRNG_rev_multi(mut res:u32,n:u32) -> u32 {
  // fp = y => ((y - 1n) * 210844021n) % 4294967296n;
  for i in 0..n {
    res = TeamRNG_rev(res)
  }
  res
}
