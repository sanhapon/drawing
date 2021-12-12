use wasm_bindgen::prelude::*;

const WASM_MEMORY_BUFFER_SIZE: usize = 2;

static mut WASM_MEMORY_BUFFER: [u8; WASM_MEMORY_BUFFER_SIZE] = [0; WASM_MEMORY_BUFFER_SIZE];

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn console_log_from_wasm() {
    log("hello from wasm");
}


#[wasm_bindgen]
pub fn store_value_in_wasm_memory_buffer_index_zero(value: u8) {
    unsafe {
        WASM_MEMORY_BUFFER[0] = value;
    }
}

#[wasm_bindgen]
pub fn get_wasm_memory_buffer_pointer() -> *const u8 {
    let pointer: *const u8;
    unsafe {
        pointer = WASM_MEMORY_BUFFER.as_ptr();
    }
    return pointer;
}

#[wasm_bindgen]
pub fn read_wasm_memory_buffer_and_return_index_one() -> u8 {
    let value: u8;
    unsafe {
        value = WASM_MEMORY_BUFFER[1];
    }
    return value;
}