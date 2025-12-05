# how

**Natural Language to Shell Command Translator.**  
Local. Private. Fast. No API keys required.

`how` converts your plain English requests into specific shell commands using a built-in, lightweight AI model. It runs entirely on your machine.

## Features
*   **100% Local:** Runs offline. Your data never leaves your terminal.
*   **Zero Config:** No OpenAI keys, no Python environment, no Docker.
*   **Smart:** Uses a fine-tuned 0.5B model optimized for CLI tasks.
*   **Safe:** Prints the command for your review (or pipes it if you're brave).

## Usage

```console
$ how "find all jpg files larger than 10MB"
find . -name "*.jpg" -size +10M
```

## Installation

### macOS (Homebrew)
```bash
brew tap hansbala/how
brew install how
```

### Linux / Manual
```bash
curl -sL https://raw.githubusercontent.com/hansbala/how/main/install.sh | bash
```

### Build from Source

Requires Rust and Clang.

```bash
cargo install --path .
```

## How it Works

`how` embeds a quantized Qwen2.5-Coder-0.5B model (~350MB) directly into the binary.

1. On the first run, it extracts the model to your system cache.
2. It loads the model into RAM (using mmap for speed).
3. It tokenizes your prompt, runs inference locally, and outputs the raw command.

## License

MIT
