FROM rust:1.88-trixie AS builder

WORKDIR /app
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin hid-oauth

FROM gcr.io/distroless/cc-debian13:debug-nonroot AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/hid-oauth /usr/local/bin/hid-oauth

ENV PORT 3000
EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/hid-oauth"]
