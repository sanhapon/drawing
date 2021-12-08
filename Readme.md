# Live Drawing Canvas
This simple webpage is hosted by warp, the webserver framework for warp speed. When clicking or moving the mouse, we will send XY coordinates to the webserver using websockets. The Webserver was developed by rust language; it maintained collections of client's connection which will boardcast incoming coordinates to all clients. We also keep coordinates in LinkedList so that when new clients connect to the server, it will get all previous drawings.

We cleaned the canvas when LinkedList (of coordinates) contain more than 2,000 messages.

## To run server
```
cargo run
```

## Run with docker
```
docker build -t drawing .
docker run -it 8000:8000 drawing
```

Open browser then navigate to http://127.0.0.1:8000, drawing screen will be displayed. Once draw a line, javascript opens websocket to
http://127.0.0.1:8000/ws
