# Docker instruction

## Random

* `docker info`
* `docker system prune -a`
* `docker images`
* `docker ps`
* `docker run --rm ai-gen /app/ai-cli list`
* `docker build --tag ai-gen --file Dockerfile`

```docker
RUN curl -o libtorch.zip -L \
  https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.0.0%2Bcpu.zip

RUN unzip libtorch.zip

docker run -it --rm ai-gen /bin/sh
docker image tag ai-gen zurgl/ai-gen
docker push zurgl/ai-gen
```

## llama

```sh
pip3 install virtualenv
virtualenv llama
source ./llama/bin/activate
pip3 install torch torchvision torchaudio safetensors
python3 convert.py consolidated.00.pth
mv llama.safetensors ./.ai-data/llama_7B/llama.safetensors
deactivate
```

## Todo

As of Rust 1.50 you can use bool::then:

```rust
assert_eq!(false.then(|| val), None);
assert_eq!(true.then(|| val), Some(val));
```

You can convert it to a Result by chaining Option::ok_or:

```rust
assert_eq!(false.then(|| val).ok_or(err), Err(err));
assert_eq!(true.then(|| val).ok_or(err), Ok(val));
```

As of Rust 1.62, you can use bool::then_some and pass a value directly instead of creating a closure:

```rust
assert_eq!(false.then_some(val), None);
assert_eq!(true.then_some(val), Some(val));
```

Alternatively, you can use Option::filter:

```rust
assert_eq!(Some(obj).filter(|_| false), None);
assert_eq!(Some(obj).filter(|_| true).ok_or(err), Ok(obj));
```
