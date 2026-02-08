FROM rust:1.78 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin rust-keyval

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/rust-keyval /app/rust-keyval
EXPOSE 6374 9100
CMD ["/app/rust-keyval"]
