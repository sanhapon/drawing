import init from "./pkg/drawing_wasm.js";

const runWasm = async () => {
    const wasm = await init("./pkg/drawing_wasm_bg.wasm");
    wasm.start();
}

runWasm();
