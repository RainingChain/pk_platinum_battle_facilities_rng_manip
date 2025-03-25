
use pk_platinum_battle_facilities_rng_manip::m::{Options,ExecObjective,exec_generate_one,exec_search_nearby, exec_find_day_seed};

fn main() {
  let mut args_vec:Vec<String> = std::env::args().collect();
  args_vec.remove(0); // remove .exe

  let opts = Options::from_verb_args_vec(args_vec);
  if opts.is_none() {
    Options::display_help();
    return;
  }
  let opts = opts.unwrap();
  if !opts.errors.is_empty() {
    panic!("{}", opts.errors[0]);
  }

  let time_at_start = std::time::SystemTime::now();

  match opts.exec_obj {
    ExecObjective::generate_one => { exec_generate_one(opts); },
    ExecObjective::search_easy => { exec_search_nearby(&opts); },
    ExecObjective::find_seeds => { exec_find_day_seed(&opts); },
    _ => { panic!() },
  };

  let dur_in_ms = std::time::SystemTime::now().duration_since(time_at_start).unwrap().as_millis();
  println!("Duration: {}ms", dur_in_ms);
}
