
# Loom – Legacy Code Modernization Toolkit

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![Platform](https://img.shields.io/badge/platform-windows%20%7C%20linux%20%7C%20macos-lightgrey)

**Loom** is a lightweight, extensible tool that translates legacy code (COBOL, RPG, PL/I, and a simple mini-language) into modern languages (Python, JavaScript, C#, Go, Rust, TypeScript, Kotlin, Swift, Zig, Nim, Dart). It validates translations using an interpreter and supports incremental migration via a strangler-fig pattern.

> ✅ Runs efficiently on **4GB RAM, i3 CPU** – no cloud or heavy dependencies .

---

## Features

| Area | Description |
|------|-------------|
| **Legacy languages** | Simple (custom), COBOL, RPG, PL/I – easily add more via hand-written or pest grammars |
| **Target languages** | 11 modern languages: Python, JavaScript, C#, Go, Rust, TypeScript, Kotlin, Swift, Zig, Nim, Dart |
| **Statements** | `ADD`, `MOVE`, `IF`/`ELSE`, `PERFORM` (subroutines), `WHILE` loops, `DISPLAY`, `EVALUATE`, file I/O (`OPEN`, `READ`, `WRITE`, `CLOSE`) |
| **Validation** | Built-in interpreter compares legacy vs. translated output; can record/load test cases |
| **Migration** | Generates strangler-fig wrappers for incremental replacement |
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
```
The binary is located at:

target/release/loom

(or loom.exe on Windows)


---

Usage

loom translate -f <legacy-file> -l <legacy-lang> -t <target-lang>
loom validate -f <legacy-file> -l <legacy-lang> [-v key=value] [--record]
loom migrate -l <legacy-file> [-m <modern-file>] -t <target-lang>


---

Supported Legacy Languages (-l)

Language	Flag	Example Extension

Simple	simple	.legacy
COBOL	cobol	.cob, .cbl
RPG	rpg	.rpg
PL/I	pli	.pli



---

Supported Target Languages (-t)

python, javascript, csharp, go, rust, typescript, kotlin, swift, zig, nim, dart


---

Examples

1. Translate a COBOL program to Python

Input (test.cob):

MOVE 5 TO X
ADD 10 TO X
DISPLAY X

Command:

loom translate -f test.cob -l cobol -t python

Output:

def translated_func():
    x = 5
    x = x + 10
    print(x)


---

2. Validate with random test cases

loom validate -f test.cob -l cobol --record

Saves test cases to:

test.cob.tests.json

Next run without --record will reuse them.


---

3. Generate a migration wrapper

loom migrate -l legacy.cob -m modern.py -t python

Outputs a Python wrapper that routes calls between the two versions.


---

Architecture

Legacy Code → [Parser] → IR → [Translator] → Modern Code
                    ↘ [Interpreter] → Validation
                    ↘ [Migration] → Wrapper

IR – language-agnostic AST (defined in ir.rs)

Interpreter – runs the IR for validation

Validation – compares interpreter output with generated code; records/loads test cases

Migration – generates strangler-fig wrappers



---

License

MIT © Tayeb Mekati – see LICENSE for details.

