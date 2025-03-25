
import { default as wasm, exec_generate_one_wasm,exec_search_nearby_wasm,exec_find_day_seed_wasm } from "./pkg/pk_platinum_battle_facilities_rng_manip.js";

globalThis.js_postprogress = function(done, todo){
  postMessage({type:'progress',data:[done, todo]});
};
globalThis.js_date_now = function(){
  return Date.now();
}

globalThis.js_printerr = function(){
  console.log(...arguments);
};

globalThis.js_log = function(){
  console.log(...arguments);
};

let promiseWasm = wasm({
  module_or_path:new URL("./pkg/pk_platinum_battle_facilities_rng_manip_bg.wasm", globalThis.location.href)
});

addEventListener("message", async (event) => {
  await promiseWasm;

  //exec_find_day_seed_wasm(opts_args:&str,min_diff_day_adv:u32,max_diff_day_adv:u32,incr:u32,printProgress:bool) -> String {
  //exec_search_nearby_wasm(opts_args:&str,min_diff_day_adv:u32,max_diff_day_adv:u32,incr:u32,printProgress:bool) -> String {
  //exec_generate_one_wasm(opts_args:&str) -> String {
  const {verb,args} = event.data;
  console.log('cargo run -- ' + verb + ' ' + args[0]);
  console.log(args.slice(1));
  let fn = (() => {
    if (verb === 'generate_one')
      return exec_generate_one_wasm;
    if (verb === 'search_easy')
      return exec_search_nearby_wasm;
    if (verb === 'find_seeds')
      return exec_find_day_seed_wasm;
  })();

  let res = JSON.parse(fn(...args));
  postMessage({type:'result',data:res});
});