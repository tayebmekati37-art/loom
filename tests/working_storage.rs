use loom::parser_cobol::parse_program;

#[test]
fn parses_working_storage() {
    let src = r#"
       01 BALANCE PIC S9(7)V99 COMP-3.
       01 NAME PIC X(30).
    "#;

    let program = parse_program(src).unwrap();

    assert_eq!(program.variables.len(), 2);
}
