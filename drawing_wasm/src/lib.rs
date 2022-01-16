// https://rustwasm.github.io/wasm-bindgen/examples/paint.html

use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use serde_json::json;
use std::f64;
use web_sys::{MessageEvent, WebSocket, CanvasRenderingContext2d};
use closure::closure;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn start(ws_location: &str) -> Result<(), JsValue> {
    log("start");
    log(ws_location);

    let ws = WebSocket::new("ws://localhost/ws")?;

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

    {
        let msg_read_fp = Box::new({
            let context = context.clone();

            move |e: MessageEvent| {
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    let json_obj : serde_json::Value = serde_json::from_str(txt.as_string().unwrap().as_str()).unwrap();

                    if json_obj["msg_type"].as_str().unwrap().eq("line") {
                        let line = &json_obj["line"];
                        let lx :f64 = line["last_x"].to_string().parse().unwrap();
                        let ly :f64 = line["last_y"].to_string().parse().unwrap();
                        let nx :f64 = line["new_x"].to_string().parse().unwrap();
                        let ny :f64 = line["new_y"].to_string().parse().unwrap();

                        draw(&context, lx, ly, nx, ny);
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

fn add_listener(
    canvas: &web_sys::HtmlCanvasElement,
    event_name: &str,
    fn_pointer: Box<dyn FnMut(web_sys::MouseEvent)>) -> Result<(), JsValue> {
        let callback= Closure::wrap(fn_pointer);
        canvas.add_event_listener_with_callback(event_name, callback.as_ref().unchecked_ref())?;
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
