Write-Host "=== ADDING SAFE COPYBOOK SUPPORT ===" -ForegroundColor Cyan

$path = "src\parser_cobol.rs"

$content = Get-Content $path -Raw

if ($content -match "WARNING: Copybook not found") {

    Write-Host "COPYBOOK support already exists." -ForegroundColor Yellow
    exit 0
}

$needle = 'let lower = line.to_lowercase();'

$insert = @'

            // COPYBOOK SUPPORT
            if lower.starts_with("copy ") {

                let copybook = line
                    .trim_end_matches(".")
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("");

                let copy_paths = vec![
                    format!("copybooks/{}.cpy", copybook),
                    format!("copybooks/{}.cob", copybook),
                    format!("copybooks/{}", copybook),
                ];

                let mut found = false;

                for path in copy_paths {

                    if std::path::Path::new(&path).exists() {

                        let content = std::fs::read_to_string(&path)?;

                        let nested = parse_program(&content)?;

                        statements.extend(nested);

                        found = true;

                        break;
                    }
                }

                if !found {
                    eprintln!("WARNING: Copybook not found: {}", copybook);
                }

                i += 1;
                continue;
            }

'@

if ($content.Contains($needle)) {

    $replacement = $needle + "`r`n" + $insert

    $content = $content.Replace($needle, $replacement)

    Set-Content $path $content -Encoding UTF8

    Write-Host "COPYBOOK support added safely." -ForegroundColor Green

} else {

    Write-Host "Could not find insertion point." -ForegroundColor Red
    exit 1
}

# Create copybooks directory
if (!(Test-Path "copybooks")) {

    New-Item -ItemType Directory -Path "copybooks" | Out-Null

    Write-Host "Created copybooks folder." -ForegroundColor Green
}

# Create sample copybook
$sample = @'
       01 CUSTOMER-ID PIC 9(10).
       01 CUSTOMER-NAME PIC X(30).
'@

Set-Content "copybooks\customer.cpy" $sample -Encoding UTF8

Write-Host "Created sample copybook." -ForegroundColor Green

cargo fmt
cargo build

if ($LASTEXITCODE -eq 0) {

    Write-Host ""
    Write-Host "=== COPYBOOK SUPPORT SUCCESS ===" -ForegroundColor Green

} else {

    Write-Host ""
    Write-Host "BUILD FAILED" -ForegroundColor Red
}