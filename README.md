# test-llama-rust

Do not use it, just for learning & testing.

Build:

    docker build github.com/angt/test-llama-rust -t llama

> [!WARNING]
> `llama.cpp` is compiled for the native CPU. It may not work on different CPUs.

Download a GGUF model:

    curl -o qwen.gguf -sSf https://huggingface.co/Qwen/Qwen2.5-3B-Instruct-GGUF/resolve/main/qwen2.5-3b-instruct-q4_0.gguf?download=true

Run:

    docker run -v $PWD/qwen.gguf:/qwen.gguf llama -m /qwen.gguf -n 2048 "Life is "
