For wasm:
  https://developer.mozilla.org/en-US/docs/WebAssembly/Guides/Rust_to_Wasm

  cargo install wasm-pack
  ./publish.cmd

    wasm-pack build --debug --target web
    wasm-pack build --target web

  npx http-server
  open http://127.0.0.1:8080/wasm_test.html

  make sure to disable cache (console -> network -> disable cache)

  Executor.debug_exec_wasm("generate_one --same_day_seed 0x15BC6A14 --facility single --wins 0")