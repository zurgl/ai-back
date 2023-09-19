# just manual: https://github.com/casey/just#readme

_default:
  just --list

build:
  cargo build

test:
  cargo test -p models --test '*' -- --nocapture
  cargo test -p actors --test '*' -- --nocapture
  cargo test -p server --test '*' -- --nocapture

docker:
  docker build --tag ai-gen --file Dockerfile .
  docker image tag ai-gen zurgl/ai-gen
  docker push zurgl/ai-gen

init:
  curl -o libtorch.zip -L \
    https://download.pytorch.org/libtorch/cu118/libtorch-cxx11-abi-shared-with-deps-2.0.0%2Bcu118.zip
  unzip libtorch.zip
  rm libtorch.zip
  mkdir images
  mkdir -pv .ai-data/stable_diffusion_2_1
  yes | cp -rf configuration/diffusion/*.ron .ai-data/stable_diffusion_2_1/

oneshot:
  cargo run --bin airs -- diffusion oneshot

serve:
  cargo run --bin airs -- serve