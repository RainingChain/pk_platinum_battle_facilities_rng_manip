
// Executor.debug_exec_wasm('search_easy --facility single --same_day_seed 0x4CB8E31 --diff_day_seed 0xcf835d7 --wins 49 --filter_min_rating 15 --max_diff_day_change 20 --max_same_day_adv 5')

// Executor.debug_exec_wasm('generate_one --same_day_seed 0x15BC6A14 --facility single --wins 0 --print_rng_frames_info false')
// Executor.debug_exec_wasm('find_seeds --facility double --wins 0 --diff_day_seed 0 --battled_pokemons Baltoy,Vulpix,Gible,Cyndaquil,Castform,Remoraid,Mudkip,Lombre,Larvitar,Shieldon,Croagunk,Onix,Loudred,Clefairy,Pidgeotto,Turtwig,Munchlax,Yanma,Ledian,Wailmer,Illumise,Murkrow,Aipom,Chatot,Cherrim,Kadabra,Marshtomp,Wormadam');


export class Executor {
  workers = [];
  onprogress = null;

  static async debug_exec_wasm(cmd){
    const exec = new Executor();
    exec.onprogress = function(progress){
      let pct = (progress[0]/progress[1]*100).toFixed(0);
      console.log(`${progress[0]}/${progress[1]} (${pct}%)`);
    };

    const res = await exec.exec_wasm(cmd);
    console.log(res);
    return res;
  };

  /** entry point */
  exec_wasm = async function(cmd){
    const {verb,map} = this.cmdToMap(cmd);
    if (verb === 'generate_one')
      return this.exec_generate_one_wasm(map);

    if (verb === 'search_easy')
      return this.exec_search_nearby_wasm(map);

    if (verb === 'find_seeds')
      return this.exec_find_day_seed_wasm(map);

    return null;
  }
  terminateAll(){
    this.workers.forEach(w => w.terminate());
    this.workers = [];
  }
  async exec_generate_one_wasm(map){
    const [worker,promise] = this.execute_singlethread('generate_one', [this.mapToOptsArgs(map)]);
    return promise;
  }
  async exec_search_nearby_wasm(map){
    let max_diff_day_adv = this.get_max_diff_day_adv(map);

    const execInfos = this.split_for_multi_thread(map, 0, max_diff_day_adv);
    const packs = execInfos.map(execInfo => {
      return this.execute_singlethread('search_easy', execInfo);
    });
    this.workers = packs.map(p => p[0]); //TODO terminate all
    const promises = packs.map(p => p[1]);
    const resultsRaw = await Promise.all(promises);
    const results = resultsRaw.flat().filter(r => r);
    if(results.length === 0)
      return null;

    return results.sort((a,b) => {
      return b.rating - a.rating;
    })[0];
  }
  async exec_find_day_seed_wasm(map){
    const execInfos = this.split_for_multi_thread(map, 0, 2**32-1);
    const packs = execInfos.map(execInfo => {
      return this.execute_singlethread('find_seeds', execInfo);
    });
    this.workers = packs.map(p => p[0]); //TODO terminate all
    const promises = packs.map(p => p[1]);
    const resultsRaw = await Promise.all(promises);
    return resultsRaw.flat().filter(r => r);
  }

  get_max_diff_day_adv(map){
    let max_diff_day_adv = map.get('--max_diff_day_adv');
    if (max_diff_day_adv !== undefined)
      return max_diff_day_adv;

    let max_diff_day_change = map.get('--max_diff_day_change');
    if (max_diff_day_change !== undefined){
      const days_2000_to_2099 = 36524;
      const max_diff_day_change_max_value = 117593;
      return max_diff_day_change > max_diff_day_change_max_value
        ? 4294967295 : max_diff_day_change * days_2000_to_2099;
    }

    return 365240;
  }

  execute_singlethread(verb, args){
    const worker = new Worker('./worker.js',{ type: "module" });
    const promise = new Promise(resolve => {
      worker.addEventListener("message", (event) => {
        if (event.data.type === 'result')
          resolve(event.data.data);
        else if (event.data.type === 'progress' && this.onprogress)
          this.onprogress(event.data.data);
      });
      worker.onerror = function(err){
        console.log(err);
      }
      worker.postMessage({verb,args});
      return worker;
    });
    return [worker, promise];
  }

  split_for_multi_thread(map, start, end){
    let count = (end - start) + 1;

    let threads_to_use = Math.min(window.navigator.hardwareConcurrency, count);

    if (threads_to_use <= 1) {
      return [[
        this.mapToOptsArgs(map),
        start,
        end,
        threads_to_use,
        true,
      ]];
    }

    let res = [];
    for (let i = 0; i < threads_to_use; i++){
      res.push([
        this.mapToOptsArgs(map),
        start + i,
        end,
        threads_to_use,
        i === 0,
      ]);
    }
    return res;
  }
  mapToOptsArgs(map){
    let str = [];
    map.forEach((val,key) => {
      str.push(key,val)
    });
    return str.join(' ');
  }
  cmdToMap(cmd){
    const [verb, ...args] = cmd.split(' ').filter(a => a);
    let map = new Map();
    for(let i = 0 ; i < args.length - 1; i += 2){
      map.set(args[i], args[i + 1]);
    }
    return {verb, map};
  }

}

class MyTests {
  async runAll(){
    await this.generate_one();
  }
  onprogress = function(progress){
    let pct = (progress[0]/progress[1]*100).toFixed(0);
    console.log(`${progress[0]}/${progress[1]} (${pct}%)`);
  };
  async generate_one(){
    const exec = new Executor();
    exec.onprogress = this.onprogress;
    const res = await exec.exec_wasm('generate_one --same_day_seed 0x15BC6A14 --facility single --wins 0 --print_rng_frames_info false');
    if(res.same_day_seed !== 364669460)
      throw new Error('MyTests::generate_one failed');
    console.log('MyTests::generate_one success!');
    return true;
  }
  async search_easy(){
    const exec = new Executor();
    exec.onprogress = this.onprogress;
    console.time('search_easy');
    const res = await exec.exec_wasm('search_easy --facility single --same_day_seed 0x4CB8E31 --diff_day_seed 0xcf835d7 --wins 49 --filter_min_rating 10 --max_diff_day_change 1 --max_same_day_adv 1');
    console.timeEnd('search_easy');
    if(res.rating !== 16)
      throw new Error('MyTests::search_easy failed');
    console.log('MyTests::search_easy success!');
    return true;
  }
  async find_seeds(){ // 94s
    const exec = new Executor();
    exec.onprogress = this.onprogress;
    console.time('find_seeds');
    const res = await exec.exec_wasm('find_seeds --facility double --wins 0 --diff_day_seed 0 --battled_pokemons Baltoy,Vulpix,Gible,Cyndaquil,Castform,Remoraid,Mudkip,Lombre,Larvitar,Shieldon,Croagunk,Onix,Loudred,Clefairy,Pidgeotto,Turtwig,Munchlax,Yanma,Ledian,Wailmer,Illumise,Murkrow,Aipom,Chatot,Cherrim,Kadabra,Marshtomp,Wormadam');
    console.timeEnd('find_seeds');
    console.log(res);
    if(res.length === 0 || res[0].same_day_seed !== 1829313114)
      throw new Error('MyTests::find_seeds failed');
    console.log('MyTests::find_seeds success!');
    return true;
  }

}
window.Executor = Executor;
window.my_tests = new MyTests();
//await window.my_tests.runAll();