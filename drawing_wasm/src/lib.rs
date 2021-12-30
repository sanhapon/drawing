// https://rustwasm.github.io/wasm-bindgen/examples/paint.html

use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
// use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use serde_json::json;

use std::f64;


#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}


#[wasm_bindgen]
pub fn start(ws_location: &str) -> Result<(), JsValue> {
    // use web_sys::console;
    // console::log_1(&"start".into());
    log("start");
    log(ws_location);

    let mut last_x = 0_f64;
    let mut last_y = 0_f64;

    let ws = web_sys::WebSocket::new("ws://localhost/ws")?;

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
                    .get_element_by_id("canvas")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .expect("Unnable to get canvas");

    let context = canvas.get_context("2d")?
                    .unwrap()
                    .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    let pressed = Rc::new(Cell::new(false));

    {
        let mousedown_fp = Box::new({
                let context = context.clone();  // because context and pressed variables will be move into clousure and we can't use it after this.
                let pressed = pressed.clone();
                let ws = ws.clone();

                move |event: web_sys::MouseEvent| {
                    let new_x = event.offset_x() as f64;
                    let new_y = event.offset_y() as f64;
                    context.begin_path();
                    context.move_to(new_x, new_y);
                    pressed.set(true);
                    if let Some(r) = send_msg(&ws, last_x, last_y, new_x, new_y) {
                        last_x = r.0;
                        last_y = r.1;
                    };
                }
        }) as Box<dyn FnMut(_)>;

        add_listener(&canvas, "mousedown", mousedown_fp)?;
    }
    {
        let mousemove_fp = Box::new({
                let context = context.clone();
                let pressed = pressed.clone();
                let ws = ws.clone();

                move |event: web_sys::MouseEvent| {
                    if pressed.get() {
                        let new_x = event.offset_x() as f64;
                        let new_y = event.offset_y() as f64;
                        context.line_to(new_x, new_y);
                        context.stroke();
                        context.begin_path();
                        context.move_to(new_x, new_y);
                        if let Some(r) = send_msg(&ws, last_x, last_y, new_x, new_y) {
                            last_x = r.0;
                            last_y = r.1;
                        };
                    }
                }
        }) as Box<dyn FnMut(_)>;
        add_listener(&canvas, "mousemove", mousemove_fp)?;
    }
    {
        let the_box = Box::new({
            let context = context.clone();
            let pressed = pressed.clone();

            move |event: web_sys::MouseEvent| {
                pressed.set(false);
                context.line_to(event.offset_x() as f64, event.offset_y() as f64);
                context.stroke();
            }
        }) as Box<dyn FnMut(_)>;
        add_listener(&canvas, "mouseup", the_box)?;
    }


    Ok(())
}

fn send_msg(ws: &web_sys::WebSocket, last_x: f64, last_y: f64, new_x: f64, new_y: f64) -> Option<(f64, f64)> {

    let msg = json!({
        "last_x": last_x,
        "last_y": last_y,
        "new_x": new_x,
        "new_y": new_y,
    });

    log(msg.to_string().as_str());
    match ws.send_with_str(msg.to_string().as_str()) {
        Ok(_) => log("ok"),
        Err(e) => log(e.as_string()?.as_str()),
    };


    Some((new_x, new_y))
}


fn add_listener(canvas: &web_sys::HtmlCanvasElement, event_name: &str, fn_pointer: Box<dyn FnMut(web_sys::MouseEvent)>) -> Result<(), JsValue> {
    let callback= Closure::wrap(fn_pointer);
    canvas.add_event_listener_with_callback(event_name, callback.as_ref().unchecked_ref())?;
    callback.forget();
Ok(())
}


// draw() {
//     // start
//     context.begin_path();
//     // Draw the outer circle.
//     context
//         .arc(75.0, 75.0, 50.0, 0.0, f64::consts::PI * 2.0)
//         .unwrap();

//     // Draw the mouth.
//     context.move_to(110.0, 75.0);
//     context.arc(75.0, 75.0, 35.0, 0.0, f64::consts::PI).unwrap();

//     // Draw the left eye.
//     context.move_to(65.0, 65.0);
//     context
//         .arc(60.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
//         .unwrap();

//     // Draw the right eye.
//     context.move_to(95.0, 65.0);
//     context
//         .arc(90.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
//         .unwrap();

//     context.stroke();
//     // end
// }