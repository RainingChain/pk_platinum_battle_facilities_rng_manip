
#[cfg(not(target_arch = "wasm32"))]
use std::time::{UNIX_EPOCH, SystemTime};

use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern { fn js_postprogress(cur:&str, todo:&str); }

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern { fn js_date_now() -> u32; }

#[cfg(target_arch = "wasm32")]
pub fn postprogress(cur:&str, todo:&str){
  unsafe { js_postprogress(cur, todo); }
}

#[cfg(target_arch = "wasm32")]
pub fn date_now() -> u64 {
  (unsafe { js_date_now() }) as u64
}

#[cfg(not(target_arch = "wasm32"))]
pub fn postprogress(_cur:&str, _todo:&str){ }

#[cfg(not(target_arch = "wasm32"))]
pub fn date_now() -> u64 {
  let start = SystemTime::now();
  start.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

pub struct ProgressDisplayer {
  progressTodo:u64,
  pub progress:u64,
  timeAtStart:u64,
  timeAtLastPrint:u64,
  progressAtLastPrintCheck:u64,
  print_progress:bool,
}

const PRINT_FREQ_IN_MS:u64 = 5000;

impl ProgressDisplayer {
  pub fn new(progressTodo:u64, print_progress:bool) -> ProgressDisplayer {
    if print_progress {
      postprogress(&format!("{}", 0),
                   &format!("{}", progressTodo));
    }
    ProgressDisplayer {
      progressTodo,
      progress:0,
      timeAtStart:date_now(),
      timeAtLastPrint:date_now(),
      progressAtLastPrintCheck:0,
      print_progress,
    }
  }
  pub fn on_progress(&mut self){
    self.progress += 1;

    if self.print_progress && self.progress - self.progressAtLastPrintCheck > 20000 {
      self.progressAtLastPrintCheck = self.progress;
      let now = date_now();
      let durSinceExecStartTime = now - self.timeAtLastPrint;
      if (durSinceExecStartTime > PRINT_FREQ_IN_MS) {
        let durSinceStart = (now - self.timeAtStart) / 1000;
        let pct = (self.progress as f64) * 100f64 / self.progressTodo as f64;
        println!("progress: {:.2}% {}/{} in {}s", pct, self.progress, self.progressTodo, durSinceStart);
        postprogress(&format!("{}", self.progress),
                     &format!("{}", self.progressTodo));
        self.timeAtLastPrint = now;
      }
    }
  }
}