import init from "./pkg/drawing_wasm.js";

const runWasm = async () => {
    const wasm = await init("./pkg/drawing_wasm_bg.wasm");

    const result = wasm.add(1,2);

    const anotherDiv = document.getElementById("anotherDiv");

    anotherDiv.textContent = `Hello world ${result}`;
}

runWasm();
