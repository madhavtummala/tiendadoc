from rust as builder
run mkdir -p /src
workdir /src
copy ./ .
run cargo build --release

from rust as base
run mkdir -p /usr/app/
workdir /usr/app/
copy --from=builder /src/target/release/tiendadoc /usr/app/tiendadoc
entrypoint ["/usr/app/tiendadoc"]