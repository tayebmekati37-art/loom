$ErrorActionPreference = "Continue"

$root = "C:\Users\Tayeb\Documents\loom"

Set-Location $root

Clear-Host

Write-Host ""
Write-Host "====================================="
Write-Host " SAFE LOOM FIXER"
Write-Host "====================================="
Write-Host ""

function Pause-Safely {
    Write-Host ""
    Read-Host "Press ENTER to continue"
}

Write-Host "=== FETCHING CLEAN VERSION ==="

git fetch origin
git reset --hard origin/main

Write-Host ""
Write-Host "=== PATCHING ir.rs ==="

$irPath = "src\ir.rs"
$ir = Get-Content $irPath -Raw

if ($ir -notmatch "enum PicType") {

$append = @'

#[derive(Debug, Clone)]
pub enum PicType {
    Numeric,
    AlphaNumeric,
    SignedNumeric,
    Decimal,
}

#[derive(Debug, Clone)]
pub enum CompType {
    Comp,
    Comp1,
    Comp2,
    Comp3,
    Comp5,
}

'@

    Add-Content $irPath $append
}

$ir = Get-Content $irPath -Raw

$ir = $ir -replace `
'Call\s*\{\s*program:\s*String,\s*\}',
@'
Call {
    program: String,
    using_args: Vec<String>,
}
'@

if ($ir -notmatch 'Raw\(String\)') {

    $ir = $ir -replace `
'(pub enum Condition\s*\{)',
@'
$1
    Raw(String),
'@
}

Set-Content $irPath $ir

Write-Host "ir.rs patched"

Write-Host ""
Write-Host "=== PATCHING parser_cobol.rs ==="

$parserPath = "src\parser_cobol.rs"

$parserContent = Get-Content $parserPath -Raw

$performPattern = '(?s)"perform"\s*=>\s*\{.*?anyhow::bail!\("Invalid PERFORM: \{\}", line\);\s*\}'

$performReplace = @'
"perform" => {

    if parts.len() >= 3 &&
       parts[1].to_lowercase() == "until" {

        let condition_string =
            parts[2..].join(" ");

        return Ok(Statement::PerformUntil {
            condition: Condition::Raw(condition_string),
            body: Vec::new(),
        });
    }

    anyhow::bail!("Invalid PERFORM: {}", line);
}
'@

$parserContent = [regex]::Replace(
    $parserContent,
    $performPattern,
    $performReplace,
    1
)

$callPattern = '(?s)"call"\s*=>\s*\{.*?\}'

$callReplace = @'
"call" => {

    if parts.len() < 2 {
        anyhow::bail!("Invalid CALL: {}", line);
    }

    return Ok(Statement::Call {
        program: parts[1].replace("\"", ""),
        using_args: Vec::new(),
    });
}
'@

$parserContent = [regex]::Replace(
    $parserContent,
    $callPattern,
    $callReplace,
    1
)

Set-Content $parserPath $parserContent

Write-Host "parser_cobol.rs patched"

Write-Host ""
Write-Host "=== FIXING pic_parser.rs ==="

$picContent = @'
use crate::ir::{CompType, PicType};

pub fn parse_pic(pic: &str) -> Option<PicType> {

    let upper = pic.to_uppercase();

    if upper.contains("9") {

        if upper.contains("V") {
            return Some(PicType::Decimal);
        }

        if upper.contains("S") {
            return Some(PicType::SignedNumeric);
        }

        return Some(PicType::Numeric);
    }

    if upper.contains("X") {
        return Some(PicType::AlphaNumeric);
    }

    None
}

pub fn parse_comp(line: &str) -> Option<CompType> {

    let upper = line.to_uppercase();

    if upper.contains("COMP-3") {
        return Some(CompType::Comp3);
    }

    if upper.contains("COMP-5") {
        return Some(CompType::Comp5);
    }

    if upper.contains("COMP-2") {
        return Some(CompType::Comp2);
    }

    if upper.contains("COMP-1") {
        return Some(CompType::Comp1);
    }

    if upper.contains("COMP") {
        return Some(CompType::Comp);
    }

    None
}
'@

Set-Content "src\pic_parser.rs" $picContent

Write-Host "pic_parser.rs fixed"

Write-Host ""
Write-Host "=== CLEANING ==="

cargo clean

Write-Host ""
Write-Host "=== BUILDING ==="

cargo build 2>&1 | Tee-Object build.log

Write-Host ""
Write-Host "Build Exit Code: $LASTEXITCODE"

if ($LASTEXITCODE -ne 0) {

    Write-Host ""
    Write-Host "====================================="
    Write-Host " BUILD FAILED"
    Write-Host "====================================="
    Write-Host ""

    Write-Host "Last 50 lines:"
    Write-Host ""

    Get-Content build.log -Tail 50

    Pause-Safely
    return
}

Write-Host ""
Write-Host "=== TESTING ==="

cargo test 2>&1 | Tee-Object test.log

Write-Host ""
Write-Host "Test Exit Code: $LASTEXITCODE"

if ($LASTEXITCODE -ne 0) {

    Write-Host ""
    Write-Host "====================================="
    Write-Host " TESTS FAILED"
    Write-Host "====================================="
    Write-Host ""

    Get-Content test.log -Tail 80

    Pause-Safely
    return
}

Write-Host ""
Write-Host "====================================="
Write-Host " ALL SUCCESSFUL"
Write-Host "====================================="

Pause-Safely