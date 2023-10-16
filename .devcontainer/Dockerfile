# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.145.1/containers/rust/.devcontainer/base.Dockerfile

FROM mcr.microsoft.com/vscode/devcontainers/rust:0-1-bullseye as development

RUN rustup update stable
RUN rustup default stable && rustup component add rustfmt && rustup component add clippy && rustup component add rust-src

RUN apt-get update \
  && apt-get -y install valgrind nasm tclsh pkg-config \
  libssl-dev libgnutls28-dev zlib1g-dev clang \
  libx264-dev libx265-dev libvorbis-dev libtheora-dev \
  libsdl2-dev cmake

RUN git clone -b v1.5.3 --depth 1 https://github.com/Haivision/srt.git \
  && cd srt \
  && cmake --disable-shared . \
  && make -j12 && make install \
  && cd ../ && rm -rf srt
