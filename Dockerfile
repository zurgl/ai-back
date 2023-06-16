# syntax = docker/dockerfile:1.4

################################################################################
#FROM ubuntu:22.04 AS base
FROM nvidia/cuda:11.8.0-base-ubuntu22.04 as base
################################################################################
FROM base AS builder

RUN set -eux; \
  apt update; \
  apt install -y --no-install-recommends \
  openssh-client git-core curl ca-certificates gcc libc6-dev \
  pkg-config libssl-dev cmake g++ \
  ;

# Install rustup
RUN set -eux; \
  curl --location --fail \
  "https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init" \
  --output rustup-init; \
  chmod +x rustup-init; \
  ./rustup-init -y --no-modify-path --default-toolchain stable; \
  rm rustup-init;
ENV PATH=${PATH}:/root/.cargo/bin

# Copy sources and build them
WORKDIR /app
ENV TORCH_CUDA_VERSION cu118
ENV LIBTORCH /app/libtorch
ENV LD_LIBRARY_PATH /app/libtorch/lib:$LD_LIBRARY_PATH
COPY . .
RUN --mount=type=cache,target=/root/.rustup \
  --mount=type=cache,target=/root/.cargo/registry \
  --mount=type=cache,target=/root/.cargo/git \  		
  --mount=type=cache,target=/app/target \
  set -eux; \
  cargo build --release; \
  objcopy /app/target/release/airs /app/airs; 


################################################################################
FROM base AS app

SHELL ["/bin/bash", "-c"]

WORKDIR /app
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates libgomp1 curl \
  # Clean up
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

ENV TORCH_CUDA_VERSION cu118
ENV LIBTORCH /app/libtorch
ENV LD_LIBRARY_PATH /app/libtorch/lib:$LD_LIBRARY_PATH

# Copy app from builder
WORKDIR /app
COPY configuration configuration
COPY .ai-data .ai-data
COPY ssl ssl
COPY --from=builder /app/airs airs
COPY --from=builder /app/libtorch libtorch

CMD ["/app/airs"]