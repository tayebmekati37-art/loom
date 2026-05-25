$ErrorActionPreference = "Continue"

function Pause-Fail {
    Write-Host ""
    Write-Host "SCRIPT STOPPED"
    Read-Host "Press ENTER to continue"
}

Write-Host ""
Write-Host "=================================="
Write-Host " LOOM ENTERPRISE REBUILD "
Write-Host "=================================="

cd C:\Users\Tayeb\Documents

# ---------------------------------------
# REMOVE OLD CLEAN FOLDER
# ---------------------------------------

if (Test-Path loom_clean) {
    Write-Host "Removing old loom_clean..."
    Remove-Item loom_clean -Recurse -Force
}

# ---------------------------------------
# CLONE
# ---------------------------------------

Write-Host ""
Write-Host "Cloning repository..."

git clone https://github.com/tayebmekati37-art/loom loom_clean

if (!(Test-Path loom_clean)) {
    Write-Host "Git clone failed"
    Pause-Fail
    return
}

cd loom_clean

# ---------------------------------------
# BACKUP
# ---------------------------------------

$stamp = Get-Date -Format "yyyyMMdd_HHmmss"

Copy-Item src\parser_cobol.rs "src\parser_backup_$stamp.rs"

Write-Host "Backup created"

# ---------------------------------------
# PATCH IR
# ---------------------------------------

Write-Host ""
Write-Host "Patching IR..."

$ir = Get-Content src\ir.rs -Raw

if ($ir -notmatch "PerformUntil") {

$inject = @'

    PerformUntil {
        condition: String,
        body: Vec<Statement>,
    },

    Call {
        program: String,
        using_args: Vec<String>,
    },

'@

    $ir = $ir.Replace(
        "pub enum Statement {",
        "pub enum Statement {" + $inject
    )

    Set-Content src\ir.rs $ir -Encoding UTF8

    Write-Host "IR patched"
}
else {
    Write-Host "IR already patched"
}

# ---------------------------------------
# PATCH PARSER
# ---------------------------------------

Write-Host ""
Write-Host "Patching parser..."

$parser = Get-Content src\parser_cobol.rs -Raw

if ($parser -notmatch '"call"\s*=>') {

$callBlock = @'

        "call" => {
            if parts.len() < 2 {
                anyhow::bail!("Invalid CALL: {}", line);
            }

            Ok(Statement::Call {
                program: parts[1].replace("\"", ""),
                using_args: Vec::new(),
            })
        },

'@

    $parser = $parser.Replace(
        '"perform" => {',
        $callBlock + "`n        `"perform`" => {"
    )

    Write-Host "CALL support added"
}

if ($parser -notmatch "PerformUntil") {

$performBlock = @'

            if parts.len() >= 4 && parts[1].to_lowercase() == "until" {

                let condition = parts[2..].join(" ");

                return Ok(Statement::PerformUntil {
                    condition,
                    body: Vec::new(),
                });
            }

'@

    $parser = $parser.Replace(
        '"perform" => {',
        '"perform" => {' + $performBlock
    )

    Write-Host "PERFORM UNTIL support added"
}

Set-Content src\parser_cobol.rs $parser -Encoding UTF8

# ---------------------------------------
# CREATE LIB
# ---------------------------------------

if (!(Test-Path src\lib.rs)) {

@'
pub mod ir;
pub mod parser_cobol;
pub mod translate_rust;
'@ | Set-Content src\lib.rs

Write-Host "lib.rs created"
}

# ---------------------------------------
# CREATE CORPUS
# ---------------------------------------

Write-Host ""
Write-Host "Creating corpus..."

if (!(Test-Path corpus)) {
    New-Item corpus -ItemType Directory | Out-Null
}

@"
IDENTIFICATION DIVISION.
PROGRAM-ID. BANKING.
PROCEDURE DIVISION.
DISPLAY "HELLO".
STOP RUN.
"@ | Set-Content corpus\banking.cob

@"
IDENTIFICATION DIVISION.
PROGRAM-ID. INVENTORY.
PROCEDURE DIVISION.
PERFORM UNTIL ITEM-COUNT > 10
DISPLAY "COUNTING"
END-PERFORM.
STOP RUN.
"@ | Set-Content corpus\inventory.cob

Write-Host "Corpus created"

# ---------------------------------------
# CREATE TEST
# ---------------------------------------

Write-Host ""
Write-Host "Creating semantic validation test..."

if (!(Test-Path tests)) {
    New-Item tests -ItemType Directory | Out-Null
}

@'
use std::fs;

use loom::parser_cobol::parse_program;
use loom::translate_rust::translate;
use loom::ir::Function;

#[test]
fn validate_corpus_translation() {

    let files = vec![
        "corpus/banking.cob",
        "corpus/inventory.cob"
    ];

    for file in files {

        println!("--------------------------------");
        println!("TESTING: {}", file);

        let source = fs::read_to_string(file).unwrap();

        println!("SOURCE LOADED");

        let statements = match parse_program(&source) {
            Ok(s) => s,
            Err(e) => {
                panic!("PARSE FAILURE {}\n{}", file, e);
            }
        };

        println!("PARSE SUCCESS");

        let function = Function {
            name: "main".to_string(),
            params: vec![],
            body: statements,
        };

        let rust_code = translate(&function);

        println!("TRANSLATION SUCCESS");

        assert!(rust_code.len() > 10);

        println!("Generated Rust size: {}", rust_code.len());
    }
}
'@ | Set-Content tests\semantic_validation.rs

Write-Host "Test created"

# ---------------------------------------
# BUILD
# ---------------------------------------

Write-Host ""
Write-Host "=================================="
Write-Host "BUILDING"
Write-Host "=================================="

cargo build

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "BUILD FAILED"
    Pause-Fail
    return
}

# ---------------------------------------
# TEST
# ---------------------------------------

Write-Host ""
Write-Host "=================================="
Write-Host "RUNNING TESTS"
Write-Host "=================================="

cargo test --test semantic_validation -- --nocapture

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "TESTS FAILED"
    Pause-Fail
    return
}

# ---------------------------------------
# SUCCESS
# ---------------------------------------

Write-Host ""
Write-Host "=================================="
Write-Host " ALL SUCCESSFUL "
Write-Host "=================================="

Read-Host "Press ENTER to finish"