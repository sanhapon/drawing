FROM rust:1.57 as builder

WORKDIR /usr/src/drawing

COPY . .

# # RUN cargo install --path .
RUN cargo build --release

FROM debian:buster-slim
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin/drawing
COPY --from=builder /usr/src/drawing/target/release .
COPY --from=builder /usr/src/drawing/index.html .

EXPOSE 8000
EXPOSE 80

CMD ["./drawing"]