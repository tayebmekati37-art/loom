# ============================================
# Random COBOL Validation Harness
# ============================================

$projectRoot = "C:\Users\Tayeb\Documents\loom"

Set-Location $projectRoot

$totalTests = 10
$passed = 0
$failed = 0

$tempDir = Join-Path $env:TEMP "loom_random_tests"

if (Test-Path $tempDir) {
    Remove-Item $tempDir -Recurse -Force
}

New-Item -ItemType Directory -Path $tempDir | Out-Null

function Generate-CobolProgram {
    param([int]$id)

    $value = Get-Random -Minimum 1 -Maximum 100

@"
IDENTIFICATION DIVISION.
PROGRAM-ID. TEST$id.

DATA DIVISION.
WORKING-STORAGE SECTION.
01 X PIC 9(4) VALUE $value.

PROCEDURE DIVISION.
    ADD 5 TO X.
    DISPLAY X.
    STOP RUN.
"@
}

for ($i = 1; $i -le $totalTests; $i++) {

    Write-Host ""
    Write-Host "Running test $i / $totalTests"

    $cobol = Generate-CobolProgram $i

    $file = "$tempDir\test_$i.cob"

    $cobol | Out-File $file -Encoding ascii

    $result = cargo run -- translate -f $file -l cobol -t rust 2>&1

    if ($LASTEXITCODE -eq 0) {

        Write-Host "PASS" -ForegroundColor Green
        $passed++
    }
    else {

        Write-Host "FAIL" -ForegroundColor Red
        $failed++
    }
}

Write-Host ""
Write-Host "===================================="
Write-Host "Validation Results"
Write-Host "===================================="

Write-Host "Passed: $passed" -ForegroundColor Green
Write-Host "Failed: $failed" -ForegroundColor Red
