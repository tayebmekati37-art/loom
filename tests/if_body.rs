use loom::ir::Statement;
use loom::parser_cobol::parse_program;

#[test]
fn parses_if_body() {
    let src = r#"
IF AGE > 18
DISPLAY "ADULT"
END-IF
"#;

    let ast = parse_program(src).unwrap();

    match &ast[0] {
        Statement::If { then_branch, .. } => {
            assert_eq!(then_branch.len(), 1);
        }
        _ => panic!("expected IF"),
    }
}
