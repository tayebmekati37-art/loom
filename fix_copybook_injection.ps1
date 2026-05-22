Write-Host "Fixing broken COPYBOOK injection..." -ForegroundColor Cyan

$path = "src\parser_cobol.rs"

$content = Get-Content $path -Raw

# Remove broken injected block
$content = [regex]::Replace(
    $content,
    '(?s)\s*// COPYBOOK SUPPORT.*?continue;\s*\}',
    '',
    1
)

Set-Content $path $content -Encoding UTF8

Write-Host "Broken COPYBOOK block removed." -ForegroundColor Green

cargo build