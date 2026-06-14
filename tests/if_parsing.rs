use loom::parser_cobol::parse_program;

#[test]
fn parses_if_condition() {
    let src = r#"
       IF AGE > 18
           DISPLAY "ADULT"
       END-IF
    "#;

    let result = parse_program(src);
    assert!(result.is_ok());
}
