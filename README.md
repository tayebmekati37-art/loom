# Create professional README.md and push to GitHub
Write-Host "Creating README.md..." -ForegroundColor Cyan

$readmeContent = @'
# Loom – Legacy Code Modernization Toolkit

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![Platform](https://img.shields.io/badge/platform-windows%20%7C%20linux%20%7C%20macos-lightgrey)

**Loom** is a lightweight, extensible tool that translates legacy code (COBOL, RPG, PL/I, and a simple mini‑language) into modern languages (Python, JavaScript, C#, Go, Rust, TypeScript, Kotlin, Swift, Zig, Nim, Dart). It validates translations using an interpreter and supports incremental migration via a strangler‑fig pattern.

> ✅ Runs efficiently on **4GB RAM, i3 CPU** – no cloud or heavy dependencies required.

---

## Features

| Area | Description |
|------|-------------|
| **Legacy languages** | Simple (custom), COBOL, RPG, PL/I – easily add more via hand‑written or pest grammars |
| **Target languages** | 11 modern languages: Python, JavaScript, C#, Go, Rust, TypeScript, Kotlin, Swift, Zig, Nim, Dart |
| **Statements** | `ADD`, `MOVE`, `IF`/`ELSE`, `PERFORM` (subroutines), `WHILE` loops, `DISPLAY`, `EVALUATE`, file I/O (`OPEN`, `READ`, `WRITE`, `CLOSE`) |
| **Validation** | Built‑in interpreter compares legacy vs. translated output; can record/load test cases |
| **Migration** | Generates strangler‑fig wrappers for incremental replacement |
| **Resource usage** | Compiled Rust binary, no garbage collector, minimal memory footprint |

---

## Installation

### Prerequisites
- [Rust](https://rustup.rs/) (1.70 or later)
- (Optional) Python 3 – for validation runner

### Build from source
```bash
git clone https://github.com/tayebmekati37-art/loom.git
cd loom
cargo build --release
