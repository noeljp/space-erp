FROM rust:1.100-bookworm AS build
WORKDIR /app
COPY . .
RUN cargo build --release --locked
FROM debian:bookworm-slim
COPY --from=build /app/target/release/space-erp /usr/local/bin/space-erp
USER 65534:65534
EXPOSE 3000
CMD ["space-erp"]
