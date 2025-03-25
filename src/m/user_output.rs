
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    let my_str = format!("Hello, {}", name.len());
    my_str
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern { fn js_printerr(msg:&str); }

#[cfg(target_arch = "wasm32")]
pub fn printerr(msg:&str){
  unsafe { js_printerr(msg); }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern { fn js_log(msg:&str); }

#[cfg(target_arch = "wasm32")]
pub fn log(msg:&str){
  unsafe { js_log(msg); }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn printerr(msg:&str){
    println!("{}", msg);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn log(msg:&str){
    println!("{}", msg);
}