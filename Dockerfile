FROM ubuntu:latest AS builder

RUN apt-get update \
 && apt-get install -y clang cmake curl git pkg-config

WORKDIR /usr/src/app

RUN git clone --depth 1 https://github.com/ggerganov/llama.cpp
WORKDIR /usr/src/app/llama.cpp

# use CMAKE_INSTALL_LIBDIR to fix pkg-config
RUN cmake -B build \
    -DCMAKE_INSTALL_PREFIX=/usr \
    -DCMAKE_INSTALL_LIBDIR=/usr/lib \
    -DCMAKE_C_COMPILER=clang \
    -DCMAKE_CXX_COMPILER=clang++ \
    -DLLAMA_BUILD_COMMON=OFF \
    -DLLAMA_BUILD_TESTS=OFF \
    -DLLAMA_BUILD_EXAMPLES=OFF \
    -DLLAMA_BUILD_SERVER=OFF \
 && cmake --build build --config Release -j
RUN cmake --install build

WORKDIR /usr/src/app

RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
ENV PATH="/root/.cargo/bin:$PATH"

COPY Cargo.toml build.rs .
COPY src src
RUN cargo build --release

FROM ubuntu:latest

COPY --from=builder /usr/src/app/target/release/test-llama-rust /usr/bin/
COPY --from=builder /usr/lib/libllama.so /usr/lib/
COPY --from=builder /usr/lib/libggml*.so /usr/lib/

ENTRYPOINT ["test-llama-rust"]
