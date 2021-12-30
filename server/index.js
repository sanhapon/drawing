// import * as wasm from "./pkg/drawing_wasm.js";

// wasm.say_hi();

import init from "./pkg/drawing_wasm.js";

const runWasm = async () => {
    const wasm = await init("./pkg/drawing_wasm_bg.wasm");
    // let ws_location = `ws://${window.location.hostname}/ws`;
    let ws_location = 'abc';
    wasm.start(ws_location);
    // wasm.say_hi();

    // const result = wasm.add(1,2);

    // const anotherDiv = document.getElementById("anotherDiv");

    // anotherDiv.textContent = `Hello world ${result}`;
}

runWasm();
