use std::fs;
use std::process::Command;

#[test]
fn fuzz_random_cobol() {
    for i in 0..100 {
        let cobol = format!(
            r#"
       IDENTIFICATION DIVISION.
       PROGRAM-ID. TEST{}.

       DATA DIVISION.
       WORKING-STORAGE SECTION.

       01 WS-X PIC 9(4).
       01 WS-Y PIC 9(4).

       PROCEDURE DIVISION.

           MOVE {} TO WS-X
           ADD {} TO WS-X
           DISPLAY WS-X
           STOP RUN.
        "#,
            i,
            i,
            i + 1
        );

        let file = format!("temp_{}.cob", i);

        fs::write(&file, cobol).unwrap();

        let output = Command::new("cargo")
            .args([
                "run",
                "--",
                "translate",
                "-f",
                &file,
                "-l",
                "cobol",
                "-t",
                "rust",
            ])
            .output()
            .unwrap();

        assert!(output.status.success(), "Fuzz test failed {}", i);

        let _ = fs::remove_file(&file);
    }
}
