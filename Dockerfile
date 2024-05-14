FROM rust:1.78.0-bookworm as build-env
LABEL maintainer="yanorei32"

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

WORKDIR /usr/src
RUN cargo new infinite-craft-logger
COPY LICENSE Cargo.toml Cargo.lock /usr/src/infinite-craft-logger/
WORKDIR /usr/src/infinite-craft-logger
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN	cargo install cargo-license && cargo license \
	--authors \
	--do-not-bundle \
	--avoid-dev-deps \
	--avoid-build-deps \
	--filter-platform "$(rustc -vV | sed -n 's|host: ||p')" \
	> CREDITS

RUN cargo build --release
COPY src/ /usr/src/infinite-craft-logger/src/
COPY assets/ /usr/src/infinite-craft-logger/assets/
RUN touch  assets/* src/* && cargo build --release

FROM debian:bookworm-slim@sha256:911821c26cc366231183098f489068afff2d55cf56911cb5b7bd32796538dfe1

WORKDIR /

COPY --chown=root:root --from=build-env \
	/usr/src/infinite-craft-logger/CREDITS \
	/usr/src/infinite-craft-logger/LICENSE \
	/usr/share/licenses/infinite-craft-logger/

COPY --chown=root:root --from=build-env \
	/usr/src/infinite-craft-logger/target/release/infinite-craft-logger \
	/usr/bin/infinite-craft-logger

VOLUME /var/infinite-craft-logger/
ENV DB_PATH /var/infinite-craft-logger/recipes.json
ENV LISTEN 0.0.0.0:8888

CMD ["/usr/bin/infinite-craft-logger"]
