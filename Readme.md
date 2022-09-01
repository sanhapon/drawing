# Live Drawing Canvas

This simple webpage is hosted by warp, the webserver framework for warp speed. When clicking or moving the mouse, we will send XY coordinates to the webserver using websockets. The Webserver was developed by rust language; it maintained collections of client's connection which will boardcast incoming coordinates to all clients. We also keep coordinates in LinkedList so that when new clients connect to the server, it will get all previous drawings.

We cleaned the canvas when LinkedList (of coordinates) contain more than 10,000 messages.

The client is developed using WASM (Web Assembly) by Rust language. The web_sys lib crate is a bridge between Html Canvas and Rust, we handle mouse down, mouse up and mouse move canvas 's event to send TCP message to the server. We also listen for the messge from other browsers to draw lines.


## Build and run local server

We draw the canvas using web assembly, lib.rs, in client-side (draing_wasm) project and use it in the html. Thus we need to build wasm first then server later. We might need to install [wasm-pack] if it is neccessary. Note, we keep wasm output in the server project.

Use the following command to install wasm-pack
```
cargo install wasm-pack
```

Now build wasm library and run server
```
$cd drawing_wasm
$wasm-pack build --release --target web --out-dir ../server/pkg
$cd ../server
$cargo run
```

## Run with docker

Or you can build and run with docker with the following command
```
docker build -t drawing .
docker run -it 80:80 drawing
```

Open 2 browsers or share with your friends, navigate to http://127.0.0.1, drawing screen will be displayed. Once draw a line, javascript opens websocket to ws://127.0.0.1/ws


