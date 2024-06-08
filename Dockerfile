FROM rust:1.76-alpine AS builder

RUN apk add --no-cache musl-dev sqlite-dev

WORKDIR /src
COPY . .

RUN cargo build --release --locked

FROM alpine:3.17 AS runner

RUN apk add --no-cache sqlite

WORKDIR /app
COPY --from=builder /src/target/release/sqlite-auth-request /app/

EXPOSE 8080

ENTRYPOINT ["/app/sqlite-auth-request"]