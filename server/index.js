import init from "./pkg/wasm_drawing.js";

const runWasm = async () => {
    const wasm = await init("./pkg/wasm_drawing_bg.wasm");

    const result = wasm.add(1,2);

    const anotherDiv = document.getElementById("anotherDiv");

    anotherDiv.textContent = `Hello world ${result}`;
}

runWasm();
