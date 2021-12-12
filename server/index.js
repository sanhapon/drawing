import wasmInit from "./pkg/drawing_wasm.js";

const runWasm = async () => {
    const rustWasm = await wasmInit("./pkg/drawing_wasm_bg.wasm");
    rustWasm.store_value_in_wasm_memory_buffer_index_zero(24);
    let wasmMemory = new Uint8Array(rustWasm.memory.buffer);
    let bufferPointer = rustWasm.get_wasm_memory_buffer_pointer();
    console.log(wasmMemory[bufferPointer + 0]);
    wasmMemory[bufferPointer + 1] = 40;
    console.log(wasmMemory[bufferPointer + 1]);
    console.log(rustWasm.read_wasm_memory_buffer_and_return_index_one());
    rustWasm.console_log_from_wasm();

}

runWasm();