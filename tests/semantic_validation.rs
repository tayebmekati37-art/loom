use std::fs;
use std::path::Path;

use loom::ir::Function;
use loom::parser_cobol::parse_program;
use loom::translate_rust::translate;

#[test]
fn validate_corpus_translation() {
    let corpus = Path::new("corpus");

    for entry in fs::read_dir(corpus).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("cob") {
            continue;
        }

        println!("--------------------------------");
        println!("TESTING: {:?}", path);

        let source = fs::read_to_string(&path).expect("Cannot read COBOL file");

        println!("SOURCE LOADED");

        let statements = match parse_program(&source) {
            Ok(s) => {
                println!("PARSE SUCCESS");
                s
            }

            Err(e) => {
                panic!("\nPARSE FAILURE\nFILE: {:?}\nERROR:\n{:?}", path, e);
            }
        };

        let function = Function {
            name: "main".to_string(),
            body: statements,
        };

        let rust_code = translate(&function);

        println!("TRANSLATION SUCCESS");
        println!("Generated Rust size: {}", rust_code.len());

        if rust_code.trim().is_empty() {
            panic!("\nEMPTY TRANSLATION\nFILE: {:?}", path);
        }
    }
}
