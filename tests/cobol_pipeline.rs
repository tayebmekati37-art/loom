// M3.1 — end-to-end COBOL -> Rust pipeline tests.
//
// These tests are the project's first proof that a COBOL source file
// flows all the way through:
//
//   parser_cobol::parse_program
//   -> translate_rust::translate_function
//   -> rustc compile
//   -> run binary
//   -> stdout compared to expected.txt
//
// Any regression in parse, translate, or codegen that breaks the
// literal-DISPLAY + STOP-RUN subset is caught here.

use loom::ir::Function;
use loom::parser_cobol;
use loom::translate_rust;
use std::process::Command;

fn translate_cobol_to_rust(source: &str) -> String {
    let statements = parser_cobol::parse_program(source).expect("COBOL parse must succeed");
    let function = Function {
        name: "translated_func".to_string(),
        body: statements,
    };
    translate_rust::translate_function(&function)
}

fn compile_and_run(rust_code: &str) -> String {
    // Wrap the translated function so the produced binary has an entry point.
    let full = format!("{}\nfn main() {{ translated_func(); }}\n", rust_code);

    let tmp = tempfile::tempdir().expect("create tempdir");
    let src_path = tmp.path().join("program.rs");
    let bin_name = if cfg!(windows) { "program.exe" } else { "program" };
    let bin_path = tmp.path().join(bin_name);

    std::fs::write(&src_path, &full).expect("write generated source");

    let compile_status = Command::new("rustc")
        .arg(&src_path)
        .arg("-o")
        .arg(&bin_path)
        .status()
        .expect("rustc must be available (cargo test implies rustc is installed)");
    assert!(
        compile_status.success(),
        "rustc failed to compile generated code:\n{}",
        full
    );

    let output = Command::new(&bin_path).output().expect("run compiled binary");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn hello_cobol_fixture_translates_and_runs() {
    let dir = env!("CARGO_MANIFEST_DIR");
    let cob_path = format!("{}/tests/cobol/hello/program.cob", dir);
    let exp_path = format!("{}/tests/cobol/hello/expected.txt", dir);

    let source = std::fs::read_to_string(&cob_path).expect("read hello/program.cob");
    let expected = std::fs::read_to_string(&exp_path)
        .expect("read hello/expected.txt")
        .trim()
        .to_string();

    let rust = translate_cobol_to_rust(&source);
    let stdout = compile_and_run(&rust);

    assert_eq!(
        stdout, expected,
        "output mismatch.\nGenerated Rust:\n{}\n", rust
    );
}

#[test]
fn display_strips_quotes() {
    let rust = translate_cobol_to_rust("DISPLAY 'HELLO'");
    assert!(
        rust.contains(r#"println!("HELLO")"#),
        "expected `println!(\"HELLO\")` in output; got:\n{}",
        rust
    );
    assert!(
        !rust.contains("'HELLO'"),
        "single quotes were not stripped; got:\n{}",
        rust
    );
}

#[test]
fn stop_run_emits_return() {
    let rust = translate_cobol_to_rust("STOP RUN");
    assert!(
        rust.contains("return;"),
        "expected `return;` for STOP RUN; got:\n{}",
        rust
    );
}