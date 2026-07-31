<div align="center">
  <h1>🧠 GitMind</h1>
  <p><em>Meaningful Git commit messages powered by Large Language Models.</em></p>
  <!-- Badges -->
  <p>
    <img src="https://img.shields.io/badge/Language-Rust-orange.svg" alt="Rust" />
    <img src="https://img.shields.io/badge/AI-Assisted-blue.svg" alt="AI Assisted" />
    <img src="https://img.shields.io/badge/Learning-Project-success.svg" alt="Learning Project" />
  </p>
</div>

---

## 📖 About The Project

**GitMind** is a tool designed to automatically generate meaningful, context-aware Git commit messages using Large Language Models (LLMs).

> **Note:** This project is being developed with AI assistance as a personal learning journey to master **Rust** and understand the inner workings of **Git**.

## 🎯 The Problem

The frustration isn't about _writing_ commit messages; it's about translating mental context into a one-line summary every single time. Often, we resort to `fix bug` or `update files`.

GitMind solves this by reading the actual code diffs, understanding the context (via LLMs), and generating a commit message that explains the _WHY_, not just the _WHAT_.

## ✨ Features

- 🤖 **AI-Powered:** Utilizes LLMs to analyze your staged changes and suggest descriptive commit messages.
- 🦀 **Written in Rust:** Fast, memory-safe, and compiled for performance.
- 📚 **Learning Focused:** Built from the ground up to understand system-level interactions and API integrations.
- 🧠 **Smart Diff Analysis:** Parses `git diff` intelligently before sending it to the LLM to avoid overwhelming the context window.
- 🌐 **Multi-LLM Support:** Will support local models (Ollama) and cloud APIs (OpenAI, Anthropic).
- 🖥️ **Interactive TUI:** A beautiful terminal user interface (built with `ratatui`) to review diffs, select files, and approve generated messages.

## 🗺️ Roadmap

- [x] **Phase 0:** Project initialization and CLI scaffolding.
- [x] **Phase 1 (v0.1 - CLI Core):** Implement `git2` status/diff reading and basic LLM integration.
- [ ] **Phase 2 (v0.2 - TUI):** Build the `ratatui` interface and file selection.
- [ ] **Phase 3 (v0.3 - Terminal Multiplexing):** Embed the local shell using `portable-pty`.

## 🚀 Getting Started

_(Currently in early development)_

```bash
# Clone the repository
git clone https://github.com/yourusername/gitmind.git
cd gitmind

# Run the CLI
cargo run -- --help
```

## 🛠️ Built With

- [Rust](https://www.rust-lang.org/)
- [Clap](https://crates.io/crates/clap) - CLI argument parsing
- [Git2](https://crates.io/crates/git2) - Git interactions
- [Tokio](https://tokio.rs/) - Async runtime
- _Future: Ratatui, Crossterm, Portable-pty_
