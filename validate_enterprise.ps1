$tests = Get-ChildItem tests\cobol -Directory

$passed = 0
$failed = 0

foreach ($test in $tests) {

    $program = Join-Path $test.FullName "program.cob"

    if (-not (Test-Path $program)) {
        continue
    }

    Write-Host ""
    Write-Host "Running $($test.Name)" -ForegroundColor Cyan

    $result = cargo run -- translate -f $program -l cobol -t rust 2>&1

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
Write-Host "=========================" -ForegroundColor Cyan
Write-Host "Passed: $passed"
Write-Host "Failed: $failed"
Write-Host "=========================" -ForegroundColor Cyan
