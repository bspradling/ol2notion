FROM rust:1.57.0 as build
RUN rustup component add rustfmt
WORKDIR /rust
COPY . .
RUN cargo build

FROM gcr.io/distroless/cc-debian10
ARG BIN=ol2notion
COPY --from=build /rust/target/debug/$BIN /app
ENTRYPOINT ["/app"]
