use loom::parser_cobol::parse_program;

#[test]
fn packed_decimal_parses() {
    let cobol = r#"
       IDENTIFICATION DIVISION.
       PROGRAM-ID. TEST.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-AMOUNT PIC S9(7)V99 COMP-3.
       PROCEDURE DIVISION.
           MOVE 100 TO WS-AMOUNT
           DISPLAY WS-AMOUNT
           STOP RUN.
    "#;

    let result = parse_program(cobol);

    match result {
    Ok(v) => println!("{:#?}", v),
    Err(e) => panic!("Parse failed: {:?}", e),
}
}

#[test]
fn decimal_pic_parses() {
    let cobol = r#"
       IDENTIFICATION DIVISION.
       PROGRAM-ID. TEST.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-PRICE PIC 9(5)V99.
       PROCEDURE DIVISION.
           MOVE 12 TO WS-PRICE
           STOP RUN.
    "#;

    let result = parse_program(cobol);

    assert!(result.is_ok());
}

