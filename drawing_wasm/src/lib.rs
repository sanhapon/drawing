// https://rustwasm.github.io/wasm-bindgen/examples/paint.html

use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use serde_json::json;
use std::f64;
use web_sys::{MessageEvent, WebSocket, CanvasRenderingContext2d};

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

    {
        let mousedown_fp = Box::new({
                let context = context.clone();
                let pressed = pressed.clone();
                let ws = ws.clone();

                move |event: web_sys::MouseEvent| {
                    let new_x = event.offset_x() as f64;
                    let new_y = event.offset_y() as f64;

                    context.begin_path();
                    context.move_to(new_x, new_y);
                    pressed.set(true);

                    last_x = new_x;
                    last_y = new_y;

                    let txt = format!("mouse down -> ({},{}) - ({},{})", last_x, last_y, new_x, new_y);
                    log(txt.as_str());

                    if let Some(r) = send_msg(&ws, last_x, last_y, new_x, new_y) {
                    //     last_x = r.0;
                    //     last_y = r.1;
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
                        let txt = format!("mouse move -> ({},{}) - ({},{})", last_x, last_y, new_x, new_y);
                        log(txt.as_str());
                    }
                }
        }) as Box<dyn FnMut(_)>;
        add_listener(&canvas, "mousemove", mousemove_fp)?;
    }
    {
        let mouseup_fp = Box::new({
            let context = context.clone();
            let pressed = pressed.clone();

            move |event: web_sys::MouseEvent| {
                pressed.set(false);
                context.line_to(event.offset_x() as f64, event.offset_y() as f64);
                context.stroke();

                last_x = event.offset_x() as f64;
                last_y = event.offset_y() as f64;
                let txt = format!("mouse up -> ({},{})", last_x, last_y);
                log(txt.as_str());
            }
        }) as Box<dyn FnMut(_)>;
        add_listener(&canvas, "mouseup", mouseup_fp)?;
    }

    {
        let msg_read_fp = Box::new({
            let context = context.clone();

            move |e: MessageEvent| {
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    // let txt.as_string().unwrap().as_str());
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

fn add_listener(canvas: &web_sys::HtmlCanvasElement, event_name: &str, fn_pointer: Box<dyn FnMut(web_sys::MouseEvent)>) -> Result<(), JsValue> {
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

    let txt = format!("send -> ({},{}) and ({},{})", last_x, last_y, new_x, new_y);
    log(txt.as_str());

    // match ws.send_with_str(msg.to_string().as_str()) {
    //     Ok(_) => log("send"),
    //     Err(e) => log(e.as_string()?.as_str()),
    // };

    Some((new_x, new_y))
}

fn draw(context: &CanvasRenderingContext2d, last_x: f64, last_y: f64, new_x: f64, new_y: f64) {

    context.begin_path();
    context.move_to(last_x, last_y);
    context.line_to(new_x, new_y);
    context.stroke();
    context.close_path();
}
