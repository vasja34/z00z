📌
На `2026-07-03` я бы строил devnet Docker на `Debian 12 (bookworm)`: builder `rust:1.90-slim-bookworm`, runtime `debian:12-slim`. Я отдельно проверил, что оба image/tag доступны через `docker manifest inspect`.

Причина простая: workspace pinned на Rust `1.90.0` в [Cargo.toml](/home/vadim/Projects/z00z/Cargo.toml:61), а текущий node Dockerfile в repo устарел и сидит на Alpine `rust:1.70-alpine` в [Dockerfile](z00z_networks/docker_nodes/Dockerfile:2). Плюс ваши tooling/scripts ожидают `apt`-пакеты вроде `pkg-config`, `libssl-dev`, `clang`, `lld`, `cmake` в [install-verification-tools.sh](/home/vadim/Projects/z00z/scripts/verification-tools/install-verification-tools.sh:824), так что для devnet Debian даст меньше трения, чем Alpine/musl. `Ubuntu 24.04 LTS` нормален как запасной вариант, если нужна именно Ubuntu userland; distroless для devnet я бы не брал из-за дебага и ops.

Источники: https://hub.docker.com/_/rust, https://www.debian.org/releases/bookworm/, https://documentation.ubuntu.com/release-notes/24.04/
