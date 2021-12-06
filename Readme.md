# To run server
```
cargo run
```

# Run with docker
```
docker build -t drawing .
docker run drawing
```

Open browser then navigate to http://127.0.0.1:8000, drawing screen will be displayed. Once draw a line, javascript opens websocket to
http://127.0.0.1:8000/ws
