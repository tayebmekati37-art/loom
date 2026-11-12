// M2.2 — tests for explicit unsupported handling in the COBOL parser.
use loom::ir::{ParsedProgram, Statement};
use loom::parser_cobol;

fn parse(src: &str) -> ParsedProgram {
    parser_cobol::parse_program_with_unsupported(src)
        .expect("parse_program_with_unsupported must not fail on valid input")
}

#[test]
fn stop_run_produces_stop_run_statement() {
    let parsed = parse("STOP RUN");
    assert!(
        parsed.statements.iter().any(|s| matches!(s, Statement::StopRun)),
        "expected Statement::StopRun; got: {:#?}", parsed.statements
    );
    assert!(
        parsed.unsupported.iter().all(|u| u.kind != "stop_run"),
        "STOP RUN should not be reported as unsupported"
    );
}

#[test]
fn bad_add_numeric_records_unsupported_and_continues() {
    let parsed = parse("ADD ABC TO X");
    assert!(
        parsed.unsupported.iter().any(|u| u.kind == "bad_numeric" && u.text == "ABC"),
        "expected bad_numeric record for 'ABC'; got: {:#?}", parsed.unsupported
    );
    assert!(
        parsed.statements.iter().any(|s| matches!(s, Statement::Add { value: 0, target } if target == "X")),
        "expected Add {{ value: 0, target: X }}; got: {:#?}", parsed.statements
    );
}

#[test]
fn unknown_statement_still_aborts() {
    let result = parser_cobol::parse_program_with_unsupported("XYZZY FOO BAR");
    assert!(result.is_err(), "unknown statement should still abort");
}

#[test]
fn empty_input_yields_empty_program() {
    let parsed = parse("");
    assert!(parsed.statements.is_empty());
    assert!(parsed.unsupported.is_empty());
}

#[test]
fn parse_program_compat_wrapper_drops_unsupported() {
    let stmts: Vec<Statement> = parser_cobol::parse_program("STOP RUN")
        .expect("parse_program must still succeed");
    assert!(
        stmts.iter().any(|s| matches!(s, Statement::StopRun)),
        "compat wrapper lost StopRun; got: {:#?}", stmts
    );
}