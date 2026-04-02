# Loom – Legacy Code Modernization Toolkit

**Loom** is a lightweight, extensible tool that translates legacy code (COBOL, RPG, PL/I, and a simple mini‑language) into modern languages (Python, JavaScript, C#, Go, Rust, TypeScript, Kotlin, Swift, Zig, Nim, Dart). It validates translations using an interpreter or real legacy execution, and supports incremental migration via a strangler‑fig pattern.

> **Runs efficiently on 4GB RAM, i3 CPU** – no cloud or heavy dependencies required.

---

## Features

- **Multi‑legacy input** – simple language, COBOL, RPG, PL/I (easily add more via pest grammars).
- **Multi‑modern output** – 11 target languages with identical IR.
- **Validation** – compares legacy interpreter output with generated code; can record test cases for regression.
- **Incremental migration** – generates wrapper code to route calls between legacy and modern components.
- **Resource‑friendly** – compiled Rust binary, no GC, minimal memory footprint.
- **Extensible** – add a new grammar or translator in under 100 lines.

---

## Installation

### Prerequisites
- Rust (1.70+)
- Python 3 (for validation runner, optional)
- (Optional) Legacy runtime – e.g., `cobc` for COBOL, `python` for simple language.

### Build from source
```bash
git clone https://github.com/yourusername/loom.git
cd loom
cargo build --release
