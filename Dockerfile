FROM rust as builder
COPY . /app
WORKDIR /app
RUN cargo build --release --no-default-features

FROM archlinux
COPY --from=builder /app/target/release/shfs /bin/shfs
VOLUME /config
VOLUME /volumes
EXPOSE 30
ENTRYPOINT ["/bin/shfs", "serve", "-C", "/config/config.json"]
