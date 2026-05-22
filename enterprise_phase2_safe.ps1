Write-Host "=== SAFE ENTERPRISE PHASE 2 ===" -ForegroundColor Cyan

$root = Get-Location
$src = Join-Path $root "src"

if (!(Test-Path $src)) {
    Write-Host "src folder not found" -ForegroundColor Red
    exit 1
}

# -----------------------------------------------------------------------------
# Backup
# -----------------------------------------------------------------------------

$backup = "backup_phase2_safe_" + (Get-Date -Format "yyyyMMdd_HHmmss")

Copy-Item $src $backup -Recurse

Write-Host "Backup created: $backup" -ForegroundColor Yellow

# -----------------------------------------------------------------------------
# Create pic_parser.rs
# -----------------------------------------------------------------------------

$picParser = @'
use crate::ir::{PicType, CompType};

pub fn parse_pic(pic: &str) -> Option<PicType> {

    let upper = pic.to_uppercase();

    if upper.starts_with("S9") {

        if upper.contains("V") {

            let scale = upper
                .split("V")
                .nth(1)
                .unwrap_or("")
                .chars()
                .filter(|c| *c == '9')
                .count();

            return Some(PicType::Decimal { scale });
        }

        return Some(PicType::SignedInteger);
    }

    if upper.starts_with("9") {

        if upper.contains("V") {

            let scale = upper
                .split("V")
                .nth(1)
                .unwrap_or("")
                .chars()
                .filter(|c| *c == '9')
                .count();

            return Some(PicType::Decimal { scale });
        }

        return Some(PicType::Integer);
    }

    if upper.starts_with("X") {

        return Some(PicType::String { length: extract_len(&upper) });
    }

    None
}

pub fn parse_comp(line: &str) -> Option<CompType> {

    let upper = line.to_uppercase();

    if upper.contains("COMP-3") {
        return Some(CompType::Comp3);
    }

    if upper.contains("COMP") {
        return Some(CompType::Comp);
    }

    None
}

fn extract_len(pic: &str) -> usize {

    if let Some(start) = pic.find("(") {

        if let Some(end) = pic.find(")") {

            return pic[start + 1..end]
                .parse::<usize>()
                .unwrap_or(1);
        }
    }

    1
}
'@

Set-Content "$src\pic_parser.rs" $picParser -Encoding UTF8

Write-Host "Created pic_parser.rs" -ForegroundColor Green

# -----------------------------------------------------------------------------
# Register module in main.rs
# -----------------------------------------------------------------------------

$mainPath = "$src\main.rs"

$main = Get-Content $mainPath -Raw

if ($main -notmatch "mod pic_parser;") {

    $main = "mod pic_parser;`r`n" + $main

    Set-Content $mainPath $main -Encoding UTF8

    Write-Host "Registered pic_parser module" -ForegroundColor Green
}

# -----------------------------------------------------------------------------
# Add enterprise types manually reminder
# -----------------------------------------------------------------------------

Write-Host ""
Write-Host "MANUAL STEP REQUIRED:" -ForegroundColor Yellow
Write-Host "Add these enums to ir.rs:"
Write-Host ""

Write-Host "pub enum PicType {"
Write-Host "    Integer,"
Write-Host "    SignedInteger,"
Write-Host "    Decimal { scale: usize },"
Write-Host "    String { length: usize },"
Write-Host "}"
Write-Host ""

Write-Host "pub enum CompType {"
Write-Host "    Comp,"
Write-Host "    Comp3,"
Write-Host "}"
Write-Host ""

# -----------------------------------------------------------------------------
# Create enterprise tests
# -----------------------------------------------------------------------------

New-Item -ItemType Directory -Force -Path "tests\enterprise" | Out-Null

$test = @'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. TESTPIC.

       DATA DIVISION.
       WORKING-STORAGE SECTION.

       01 CUSTOMER-ID PIC 9(10).
       01 BALANCE PIC S9(5)V99 COMP-3.

       PROCEDURE DIVISION.

           MOVE 100 TO CUSTOMER-ID
           DISPLAY CUSTOMER-ID
           STOP RUN.
'@

Set-Content "tests\enterprise\pic_test.cob" $test -Encoding UTF8

Write-Host "Created enterprise test" -ForegroundColor Green

# -----------------------------------------------------------------------------
# UTF8 normalize
# -----------------------------------------------------------------------------

Get-ChildItem $src -Filter *.rs | ForEach-Object {

    $content = Get-Content $_.FullName -Raw

    [System.IO.File]::WriteAllText(
        $_.FullName,
        $content,
        [System.Text.UTF8Encoding]::new($false)
    )
}

Write-Host "UTF8 normalized" -ForegroundColor Green

# -----------------------------------------------------------------------------
# Build
# -----------------------------------------------------------------------------

cargo build

if ($LASTEXITCODE -eq 0) {

    Write-Host ""
    Write-Host "=== SUCCESS ===" -ForegroundColor Green

} else {

    Write-Host ""
    Write-Host "BUILD FAILED" -ForegroundColor Red
    Write-Host "Restore from: $backup"
}