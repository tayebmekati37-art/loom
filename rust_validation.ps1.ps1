# Rust‑only validation harness for COBOL → Rust translator
$projectRoot = "C:\Users\Tayeb\Documents\loom"
Set-Location $projectRoot

Write-Host "Rust‑only Validation Harness (no GnuCOBOL required)" -ForegroundColor Cyan

$cobolTemplate = @'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. TESTPROG.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-INPUT  PIC S9(5)V99 VALUE __INPUT__.
       01 WS-OUTPUT PIC S9(5)V99.
       01 WS-TABLE.
          05 WS-ELEM PIC S9(3) OCCURS 5.
       PROCEDURE DIVISION.
           COMPUTE WS-OUTPUT = WS-INPUT * 2
           MOVE 100 TO WS-ELEM(3)
           DISPLAY WS-OUTPUT
           DISPLAY WS-ELEM(3).
'@

$baseCobol = $cobolTemplate -replace "__INPUT__", "0"
$baseCobolFile = Join-Path $projectRoot "base_prog.cob"
[System.IO.File]::WriteAllBytes($baseCobolFile, [System.Text.Encoding]::ASCII.GetBytes($baseCobol))

Write-Host "Translating base COBOL to Rust..." -ForegroundColor Yellow
$rustOutput = & .\target\release\loom.exe translate -f $baseCobolFile -l cobol -t rust 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "Translation failed: $rustOutput" -ForegroundColor Red
    exit 1
}
$rustCode = $rustOutput -join "`n"

$testInputs = @(10, -5, 0, 123, -99, 42)
$results = @()

foreach ($inputVal in $testInputs) {
    Write-Host "Testing input: ${inputVal}" -ForegroundColor Cyan

    $customRust = $rustCode -replace 'let mut WS-INPUT: i64 = 0;', "let mut WS-INPUT: i64 = ${inputVal};"
    $tempDir = Join-Path $env:TEMP "loom_test_${inputVal}"
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    $generatedRs = Join-Path $tempDir "generated.rs"
    $customRust | Out-File -FilePath $generatedRs -Encoding utf8

    $mainRs = @"
mod generated;

fn main() {
    generated::translated_func().unwrap();
}
"@
    $mainRsFile = Join-Path $tempDir "main.rs"
    $mainRs | Out-File -FilePath $mainRsFile -Encoding utf8

    $cargoToml = Join-Path $tempDir "Cargo.toml"
    @"
[package]
name = "loom_test"
version = "0.1.0"
edition = "2021"

[dependencies]
rust_decimal = "1.34"
"@ | Out-File -FilePath $cargoToml -Encoding utf8

    Push-Location $tempDir
    $build = & cargo build --release 2>&1
    $buildSuccess = $LASTEXITCODE -eq 0
    Pop-Location

    if (-not $buildSuccess) {
        Write-Host "Rust build failed for input ${inputVal}: $build" -ForegroundColor Red
        $results += "FAIL"
        Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
        continue
    }

    $rustExe = Join-Path $tempDir "target\release\loom_test.exe"
    $rustOutput = & $rustExe 2>&1

    $expectedFirst = $inputVal * 2
    $expectedSecond = 100
    $expectedOutput = @($expectedFirst.ToString(), "100")

    $rustLines = $rustOutput -split "`r`n" | Where-Object { $_ -ne "" }

    if ($rustLines.Count -ge 2 -and $rustLines[0] -eq $expectedFirst -and $rustLines[1] -eq "100") {
        Write-Host "Input: ${inputVal} -> PASSED" -ForegroundColor Green
        $results += "PASS"
    } else {
        Write-Host "Input: ${inputVal} -> FAILED" -ForegroundColor Red
        Write-Host "  Expected output: $($expectedOutput -join '; ')"
        Write-Host "  Rust output: $($rustLines -join '; ')"
        $results += "FAIL"
    }

    Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
}

$passed = ($results | Where-Object { $_ -eq "PASS" }).Count
$total = $results.Count
Write-Host "`nValidation summary: $passed / $total passed" -ForegroundColor $(if ($passed -eq $total) { "Green" } else { "Red" })