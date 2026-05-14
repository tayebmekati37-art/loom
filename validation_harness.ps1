# Validation harness for COBOL → Rust translator (no ACCEPT)
$projectRoot = "C:\Users\Tayeb\Documents\loom"
Set-Location $projectRoot

Write-Host "COBOL → Rust Validation Harness" -ForegroundColor Cyan

# Check GnuCOBOL
try {
    $cobcVersion = & cobc --version 2>$null
    if (-not $cobcVersion) { throw "cobc not found" }
    Write-Host "GnuCOBOL found." -ForegroundColor Green
} catch {
    Write-Host "GnuCOBOL (cobc) not found. Install it and add to PATH." -ForegroundColor Red
    exit 1
}

# Template COBOL program with placeholder for input value
$cobolTemplate = @'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. TESTPROG.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-INPUT  PIC S9(5)V99 VALUE 0.
       01 WS-OUTPUT PIC S9(5)V99.
       01 WS-TABLE.
          05 WS-ELEM PIC S9(3) OCCURS 5.
       PROCEDURE DIVISION.
           MOVE __INPUT__ TO WS-INPUT
           COMPUTE WS-OUTPUT = WS-INPUT * 2
           MOVE 100 TO WS-ELEM(3)
           DISPLAY WS-OUTPUT
           DISPLAY WS-ELEM(3)
           STOP RUN.
'@

# Translate base COBOL to Rust (without input value)
# We'll use the same template to generate Rust? Actually we translate a generic program once,
# but the Rust translation will have a variable `WS-INPUT` that we can later assign via a wrapper.
# Simpler: we will generate a base Rust code from a generic COBOL that has `MOVE WS-INPUT` but WS-INPUT is a variable.
# Then for each test, we embed the value into the Rust code (replace the initialisation of WS-INPUT) as before.
# For COBOL, we will create a temporary COBOL program per input by replacing the placeholder and compile it.

# Translate a generic COBOL program to Rust (one that declares WS-INPUT but does not initialise it with a specific value)
$genericCobol = @'
       IDENTIFICATION DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-INPUT  PIC S9(5)V99.
       01 WS-OUTPUT PIC S9(5)V99.
       01 WS-TABLE.
          05 WS-ELEM PIC S9(3) OCCURS 5.
       PROCEDURE DIVISION.
           MOVE 0 TO WS-INPUT  (placeholder)
           COMPUTE WS-OUTPUT = WS-INPUT * 2
           MOVE 100 TO WS-ELEM(3)
           DISPLAY WS-OUTPUT
           DISPLAY WS-ELEM(3)
           STOP RUN.
'@
# But the above still needs a MOVE. We'll generate a proper template.

# Better: Use the same template for both Rust and COBOL generation.
$cobolTemplate = @'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. TESTPROG.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-INPUT  PIC S9(5)V99.
       01 WS-OUTPUT PIC S9(5)V99.
       01 WS-TABLE.
          05 WS-ELEM PIC S9(3) OCCURS 5.
       PROCEDURE DIVISION.
           MOVE __INPUT__ TO WS-INPUT
           COMPUTE WS-OUTPUT = WS-INPUT * 2
           MOVE 100 TO WS-ELEM(3)
           DISPLAY WS-OUTPUT
           DISPLAY WS-ELEM(3)
           STOP RUN.
'@

# Translate a base COBOL program (with a placeholder) to Rust. We'll do this once.
$baseCobol = $cobolTemplate -replace '__INPUT__', '0'   # temporary
$cobolFile = Join-Path $projectRoot "temp_base.cob"
[System.IO.File]::WriteAllBytes($cobolFile, [System.Text.Encoding]::ASCII.GetBytes($baseCobol))

Write-Host "Translating base COBOL to Rust..." -ForegroundColor Yellow
$rustOutput = & .\target\release\loom.exe translate -f $cobolFile -l cobol -t rust 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "Translation failed: $rustOutput" -ForegroundColor Red
    exit 1
}
$rustCode = $rustOutput -join "`n"

# Remove temporary file
Remove-Item $cobolFile -ErrorAction SilentlyContinue

$testInputs = @(10, -5, 0, 123, -99, 42)
$results = @()

foreach ($inputVal in $testInputs) {
    # Generate COBOL program with the input value
    $cobolCode = $cobolTemplate -replace '__INPUT__', $inputVal
    $cobolFile = Join-Path $projectRoot "test_prog_$inputVal.cob"
    [System.IO.File]::WriteAllBytes($cobolFile, [System.Text.Encoding]::ASCII.GetBytes($cobolCode))

    # Compile COBOL
    $cobolExe = Join-Path $projectRoot "test_prog_$inputVal.exe"
    $compileCobol = & cobc -x $cobolFile -o $cobolExe 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "COBOL compilation failed for input $inputVal: $compileCobol" -ForegroundColor Red
        $results += "FAIL"
        Remove-Item $cobolFile -ErrorAction SilentlyContinue
        continue
    }

    # Run COBOL
    $cobolOutput = & $cobolExe 2>&1
    $cobolLines = $cobolOutput -split "`r`n" | Where-Object { $_ -ne "" }

    # Generate Rust code with embedded input (replace the initialisation of WS-INPUT)
    # The generated Rust code has a line like `let mut WS-INPUT: i64 = 0;`
    $customRust = $rustCode -replace '(let mut WS-INPUT: i64 = )\d+;', "`$1$inputVal;"
    # Create a temporary Rust project
    $tempDir = Join-Path $env:TEMP "loom_val_$([System.Guid]::NewGuid())"
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
    $srcDir = Join-Path $tempDir "src"
    New-Item -ItemType Directory -Path $srcDir -Force | Out-Null
    $generatedRs = Join-Path $srcDir "generated.rs"
    $customRust | Out-File -FilePath $generatedRs -Encoding utf8
    $mainRs = @"
mod generated;

fn main() {
    generated::translated_func().unwrap();
}
"@
    $mainRs | Out-File -FilePath (Join-Path $srcDir "main.rs") -Encoding utf8
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
        Write-Host "Rust build failed for input $inputVal: $build" -ForegroundColor Red
        $results += "FAIL"
        Remove-Item $cobolFile, $cobolExe -ErrorAction SilentlyContinue
        Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
        continue
    }

    $rustExe = Join-Path $tempDir "target\release\loom_test.exe"
    $rustOutput = & $rustExe 2>&1
    $rustLines = $rustOutput -split "`r`n" | Where-Object { $_ -ne "" }

    if ($cobolLines -join "," -eq $rustLines -join ",") {
        Write-Host "Input: $inputVal -> PASSED" -ForegroundColor Green
        $results += "PASS"
    } else {
        Write-Host "Input: $inputVal -> FAILED" -ForegroundColor Red
        Write-Host "  COBOL output: $($cobolLines -join '; ')"
        Write-Host "  Rust output: $($rustLines -join '; ')"
        $results += "FAIL"
    }

    # Cleanup
    Remove-Item $cobolFile, $cobolExe -ErrorAction SilentlyContinue
    Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
}

$passed = ($results | Where-Object { $_ -eq "PASS" }).Count
$total = $results.Count
Write-Host "`nValidation summary: $passed / $total passed" -ForegroundColor $(if ($passed -eq $total) { "Green" } else { "Red" })