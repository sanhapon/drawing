// https://rustwasm.github.io/wasm-bindgen/examples/paint.html

use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use serde_json::json;
use std::f64;
use web_sys::{MessageEvent, WebSocket, CanvasRenderingContext2d, HtmlButtonElement, HtmlCanvasElement}; // Added HtmlButtonElement, HtmlCanvasElement
use closure::closure;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    log("start");

    let location = web_sys::window().unwrap().location();
    let hostname = format!("ws://{}/ws", location.hostname().unwrap());
    log(&hostname[..]);

    let ws = WebSocket::new(&hostname[..])?;

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
                    .get_element_by_id("canvas")
                    .unwrap()
                    .dyn_into::<HtmlCanvasElement>() // Used HtmlCanvasElement
                    .expect("Unable to get canvas");

    let context = canvas.get_context("2d")?
                    .unwrap()
                    .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    let clear_button = document
        .get_element_by_id("clearButton")
        .unwrap()
        .dyn_into::<HtmlButtonElement>() // Added for clear button
        .expect("Unable to get clearButton");

    let pressed = Rc::new(Cell::new(false));
    let last_x = Rc::new(Cell::new(0_f64));
    let last_y = Rc::new(Cell::new(0_f64));

    {
        let context = context.clone();
        let pressed = pressed.clone();
        let last_x = last_x.clone();
        let last_y = last_y.clone();
        let ws = ws.clone();

        let mouse_move_callback = closure!(|event: web_sys::MouseEvent| {
                if pressed.get() {
                    let new_x = event.offset_x() as f64;
                    let new_y = event.offset_y() as f64;

                    context.line_to(new_x, new_y);
                    context.stroke();
                    context.begin_path();
                    context.move_to(new_x, new_y);

                    send_msg(&ws, last_x.get(), last_y.get(), new_x, new_y);

                    last_x.set(new_x);
                    last_y.set(new_y);
                }
        });

        add_listener(&canvas, "mousemove", Box::new(mouse_move_callback) as Box<dyn FnMut(_)>)?;
    }

    {
        let context = context.clone();
        let pressed = pressed.clone();
        let last_x = last_x.clone();
        let last_y = last_y.clone();

        let mouse_down_call_back = closure!(|event: web_sys::MouseEvent| {
            let new_x = event.offset_x() as f64;
            let new_y = event.offset_y() as f64;

            context.begin_path();
            context.move_to(new_x, new_y);
            pressed.set(true);

            last_x.set(new_x);
            last_y.set(new_y);
        });

        add_listener(&canvas, "mousedown", Box::new(mouse_down_call_back) as Box<dyn FnMut(web_sys::MouseEvent)>)?;
    }

    {
        let context = context.clone();
        let pressed = pressed.clone();

        let mouse_up_callback = closure!(|event: web_sys::MouseEvent| {
            pressed.set(false);
            context.line_to(event.offset_x() as f64, event.offset_y() as f64);
            context.stroke();
        });

        add_listener(&canvas, "mouseup", Box::new(mouse_up_callback) as Box<dyn FnMut(_)>)?;
    }

    // Event listener for the clear button
    {
        let ws_clone_for_clear = ws.clone();
        let clear_button_callback = closure!(|| {
            log("Clear button clicked");
            let clear_msg = json!({
                "msg_type": "clear"
            });
            ws_clone_for_clear.send_with_str(&clear_msg.to_string()).unwrap();
        });
        add_click_listener_to_button(&clear_button, Box::new(clear_button_callback) as Box<dyn FnMut()>)_?;
    }

    {
        let msg_read_fp = Box::new({
            let context = context.clone(); 
            let canvas_clone_for_onmessage = canvas.clone(); // Clone canvas for onmessage

            move |e: MessageEvent| {
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    let json_str = txt.as_string().unwrap_or_default();
                    match serde_json::from_str::<serde_json::Value>(&json_str) {
                        Ok(json_obj) => {
                            if let Some(msg_type) = json_obj["msg_type"].as_str() {
                                if msg_type.eq("line") {
                                    if let Some(line) = json_obj.get("line") {
                                        let lx :f64 = line["last_x"].as_f64().unwrap_or(0.0);
                                        let ly :f64 = line["last_y"].as_f64().unwrap_or(0.0);
                                        let nx :f64 = line["new_x"].as_f64().unwrap_or(0.0);
                                        let ny :f64 = line["new_y"].as_f64().unwrap_or(0.0);
                                        draw(&context, lx, ly, nx, ny);
                                    }
                                } else if msg_type.eq("clear") {
                                    log("Received clear message, clearing canvas.");
                                    clear_canvas(&context, &canvas_clone_for_onmessage);
                                }
                            }
                        }
                        Err(err) => {
                            log(&format!("Failed to parse JSON: {}", err));
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>;

        let onmessage_callback = Closure::wrap(msg_read_fp);

        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
    }

    Ok(())
}

// Adapted for HtmlCanvasElement
fn add_listener(
    canvas: &HtmlCanvasElement,
    event_name: &str,
    fn_pointer: Box<dyn FnMut(web_sys::MouseEvent)>) -> Result<(), JsValue> {
        let callback = Closure::wrap(fn_pointer);
        canvas.add_event_listener_with_callback(event_name, callback.as_ref().unchecked_ref())?;
        callback.forget();
        Ok(())
}

// New helper function for button click listener
fn add_click_listener_to_button(
    button: &HtmlButtonElement,
    fn_pointer: Box<dyn FnMut()>) -> Result<(), JsValue> {
        let callback = Closure::wrap(fn_pointer);
        button.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())?;
        callback.forget();
        Ok(())
}

fn send_msg(ws: &WebSocket, last_x: f64, last_y: f64, new_x: f64, new_y: f64) -> Option<(f64, f64)> {
    let msg = json!({
        "last_x": last_x,
        "last_y": last_y,
        "new_x": new_x,
        "new_y": new_y,
    });

    ws.send_with_str(msg.to_string().as_str()).unwrap();

    Some((new_x, new_y))
}

fn draw(context: &CanvasRenderingContext2d, last_x: f64, last_y: f64, new_x: f64, new_y: f64) {
    context.begin_path();
    context.move_to(last_x, last_y);
    context.line_to(new_x, new_y);
    context.stroke();
    context.close_path();
}

// Function to clear the canvas
fn clear_canvas(context: &CanvasRenderingContext2d, canvas: &HtmlCanvasElement) {
    log(&format!("Clearing canvas with dimensions: {}x{}", canvas.width(), canvas.height()));
    context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    context.begin_path(); // Reset the current path
}
