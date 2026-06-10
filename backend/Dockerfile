# syntax=docker/dockerfile:1

FROM rust:1.75-alpine3.19 AS builder

RUN apk add --no-cache musl-dev postgresql-dev openssl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY .env.example .env

RUN cargo build --release

FROM alpine:3.19

RUN apk add --no-cache ca-certificates tzdata

WORKDIR /app

COPY --from=builder /app/target/release/silkroad-analysis /app/silkroad-analysis

COPY ../frontend /app/frontend

ENV TZ=Asia/Shanghai

EXPOSE 3000

CMD ["./silkroad-analysis"]
