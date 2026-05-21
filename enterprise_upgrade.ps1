$ErrorActionPreference = "Stop"

Write-Host "=== ENTERPRISE COBOL PARSER UPGRADE ===" -ForegroundColor Cyan

$src = ".\src"

# -------------------------------------------------
# Backup
# -------------------------------------------------

$backup = "backup_enterprise_$(Get-Date -Format 'yyyyMMdd_HHmmss')"
New-Item -ItemType Directory -Path $backup | Out-Null

Copy-Item "$src\parser_cobol.rs" "$backup\parser_cobol.rs.bak"
Copy-Item "$src\ir.rs" "$backup\ir.rs.bak"

Write-Host "Backup created: $backup" -ForegroundColor Green

# -------------------------------------------------
# Add advanced IR nodes
# -------------------------------------------------

$irFile = "$src\ir.rs"
$ir = Get-Content $irFile -Raw

if ($ir -notmatch "PerformUntil") {

$insert = @"

    PerformUntil {
        condition: Condition,
        body: Vec<Statement>,
    },

    PerformVarying {
        variable: String,
        from: i64,
        by: i64,
        until: Condition,
        body: Vec<Statement>,
    },

    Call {
        program: String,
    },

    Paragraph {
        name: String,
        body: Vec<Statement>,
    },

    Section {
        name: String,
        body: Vec<Statement>,
    },

"@

$ir = $ir -replace "pub enum Statement \{", "pub enum Statement {$insert"

Set-Content $irFile $ir -Encoding UTF8

Write-Host "Added enterprise IR nodes" -ForegroundColor Green
}

# -------------------------------------------------
# Add parser support
# -------------------------------------------------

$parserFile = "$src\parser_cobol.rs"
$parser = Get-Content $parserFile -Raw

# PERFORM UNTIL
if ($parser -notmatch "perform until") {

$performUntil = @"

                "perform" => {

                    if parts.len() >= 3 -and parts[1].to_lowercase() -eq "until" {

                        let condition_str = line[13..].trim();
                        let condition = parse_condition_str(condition_str)?;

                        i += 1;

                        let mut body = Vec::new();

                        while i < lines.len() {

                            let l = lines[i].trim();

                            if l.to_lowercase().starts_with("end-perform") {
                                i += 1;
                                break;
                            }

                            if !l.is_empty() {
                                body.push(parse_single_statement(l)?);
                            }

                            i += 1;
                        }

                        statements.push(
                            Statement::PerformUntil {
                                condition,
                                body,
                            }
                        );

                    } else {

                        if parts.len() != 2 {
                            anyhow::bail!("Invalid PERFORM: {}", line);
                        }

                        statements.push(
                            Statement::Perform {
                                name: parts[1].to_string()
                            }
                        );

                        i += 1;
                    }
                }

"@

$parser = $parser -replace '"perform" => \{[\s\S]*?i \+= 1;\s*\}', $performUntil

Write-Host "Added PERFORM UNTIL support" -ForegroundColor Green
}

# -------------------------------------------------
# Add CALL support
# -------------------------------------------------

if ($parser -notmatch '"call" =>') {

$callSupport = @"

                "call" => {

                    if parts.len() < 2 {
                        anyhow::bail!("Invalid CALL: {}", line);
                    }

                    statements.push(
                        Statement::Call {
                            program: parts[1]
                                .trim_matches('"')
                                .trim_matches('\'')
                                .to_string(),
                        }
                    );

                    i += 1;
                }

"@

$parser = $parser -replace '"display" => \{', "$callSupport`n                `"display`" => {"

Write-Host "Added CALL support" -ForegroundColor Green
}

# -------------------------------------------------
# Add paragraph detection
# -------------------------------------------------

if ($parser -notmatch "Paragraph") {

$paragraphLogic = @"

            if line.ends_with(".") &&
               !line.contains(" ") &&
               !line.to_lowercase().contains("division")
            {

                statements.push(
                    Statement::Paragraph {
                        name: line.trim_end_matches(".").to_string(),
                        body: Vec::new(),
                    }
                );

                i += 1;
                continue;
            }

"@

$parser = $parser -replace 'while i < lines.len\(\) \{', "while i < lines.len() {`n$paragraphLogic"

Write-Host "Added paragraph support" -ForegroundColor Green
}

# -------------------------------------------------
# Fix encoding issues
# -------------------------------------------------

$clean = [System.Text.Encoding]::UTF8.GetString(
    [System.Text.Encoding]::Default.GetBytes($parser)
)

Set-Content $parserFile $clean -Encoding UTF8

Write-Host "UTF-8 normalized" -ForegroundColor Green

# -------------------------------------------------
# Format
# -------------------------------------------------

cargo fmt

# -------------------------------------------------
# Build
# -------------------------------------------------

cargo build

if ($LASTEXITCODE -eq 0) {

    Write-Host ""
    Write-Host "SUCCESS: Enterprise parser upgrade complete" -ForegroundColor Green
    Write-Host ""

} else {

    Write-Host ""
    Write-Host "Build failed. Restore from:" -ForegroundColor Red
    Write-Host $backup
}