FROM rust:1.57 as builder

WORKDIR /usr/src/drawing

RUN apt-get update && \
    apt-get install libssl-dev && \
    cargo install wasm-pack --no-default-features

COPY . .

WORKDIR /usr/src/drawing/server
# RUN cargo install --path .
RUN cargo build --release

# Build wasm lib
WORKDIR /usr/src/drawing/drawing_wasm
RUN wasm-pack build --release --target web --out-dir ../server/pkg

FROM debian:buster-slim

WORKDIR /usr/local/bin/drawing

RUN ls -al .

COPY --from=builder /usr/src/drawing/server/target/release .
COPY --from=builder /usr/src/drawing/server/index.html .
COPY --from=builder /usr/src/drawing/server/index.js .
COPY --from=builder /usr/src/drawing/server/pkg ./pkg

EXPOSE 8000
EXPOSE 80

CMD ["./drawing"]
