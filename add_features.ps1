@'
$projectRoot = "C:\Users\Tayeb\Documents\loom"
Set-Location $projectRoot
$utf8NoBom = New-Object System.Text.UTF8Encoding $false

Write-Host "Adding ACCEPT, STOP RUN, CONTINUE, EXIT, and INSPECT stub..." -ForegroundColor Cyan

# Update ir.rs
$irFile = Join-Path $projectRoot "src/ir.rs"
$irContent = Get-Content $irFile -Raw
if ($irContent -notmatch 'Accept { target: String }') {
    $irContent = $irContent -replace '(\n\s*\}\s*)$', @'
    Accept { target: String },
    StopRun,
    Continue,
    Exit,
    Inspect { source: String, target: String, pattern: String },
$1'@
    [System.IO.File]::WriteAllText($irFile, $irContent, $utf8NoBom)
}

# Update parser_cobol.rs
$parserFile = Join-Path $projectRoot "src/parser_cobol.rs"
$parserContent = Get-Content $parserFile -Raw
$newArms = @'
            "accept" => {
                if parts.len() < 2 {
                    anyhow::bail!("Invalid ACCEPT: {}", line);
                }
                let target = parts[1].to_string();
                statements.push(Statement::Accept { target });
                i += 1;
            }
            "stop" => {
                statements.push(Statement::StopRun);
                i += 1;
            }
            "continue" => {
                statements.push(Statement::Continue);
                i += 1;
            }
            "exit" => {
                statements.push(Statement::Exit);
                i += 1;
            }
            "inspect" => {
                eprintln!("INSPECT not fully implemented, ignoring: {}", line);
                statements.push(Statement::Display { value: crate::ir::Literal::String(format!("# INSPECT not implemented: {}", line)) });
                i += 1;
            }
'@
$parserContent = $parserContent -replace '(?s)(\s+"unstring" => \{.*?\n\s+\})', "$1`n$newArms"
[System.IO.File]::WriteAllText($parserFile, $parserContent, $utf8NoBom)

# Update translate_rust.rs
$rustFile = Join-Path $projectRoot "src/translate_rust.rs"
$rustContent = Get-Content $rustFile -Raw
$rustArms = @'
        Statement::Accept { target } => {
            writeln!(out, "{}let mut input = String::new();", indent).unwrap();
            writeln!(out, "{}{} = std::io::stdin().read_line(&mut input).unwrap();", indent, target).unwrap();
            writeln!(out, "{}{} = input.trim().parse().unwrap();", indent, target).unwrap();
        }
        Statement::StopRun => {
            writeln!(out, "{}return Ok(());", indent).unwrap();
        }
        Statement::Continue => {
            writeln!(out, "{}// CONTINUE (no-op)", indent).unwrap();
        }
        Statement::Exit => {
            writeln!(out, "{}// EXIT (no-op)", indent).unwrap();
        }
        Statement::Inspect { source, target, pattern } => {
            writeln!(out, "{}{} = {}.matches('{}').count();", indent, target, source, pattern).unwrap();
        }
'@
$rustContent = $rustContent -replace '(Statement::Compute \{ target, expr \} => \{.*?\n\s+\})', "$1`n$rustArms"
[System.IO.File]::WriteAllText($rustFile, $rustContent, $utf8NoBom)

# Create test file
$testCobol = @'
       IDENTIFICATION DIVISION.
       PROCEDURE DIVISION.
           ACCEPT WS-X
           CONTINUE
           EXIT
           STOP RUN.
'@
$testFile = Join-Path $projectRoot "test_new_features.cob"
[System.IO.File]::WriteAllBytes($testFile, [System.Text.Encoding]::ASCII.GetBytes($testCobol))

cargo clean
cargo build --release
if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful. Testing..." -ForegroundColor Green
    .\target\release\loom.exe translate -f test_new_features.cob -l cobol -t rust
} else {
    Write-Host "Build failed" -ForegroundColor Red
}
'@ | Out-File -FilePath add_features.ps1 -Encoding utf8
.\add_features.ps1