Write-Host "=== FULL PARSER RECOVERY + PATCH ===" -ForegroundColor Cyan

try {

    # FIND BACKUPS
    $backups = Get-ChildItem backup_*.rs -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending

    if ($backups.Count -eq 0) {
        Write-Host "No backups found." -ForegroundColor Red
        return
    }

    Write-Host ""
    Write-Host "AVAILABLE BACKUPS:"
    Write-Host ""

    for ($i = 0; $i -lt $backups.Count; $i++) {
        Write-Host "[$i] $($backups[$i].Name)"
    }

    Write-Host ""
    $choice = Read-Host "Choose backup number"

    if ($choice -notmatch '^\d+$') {
        Write-Host "Invalid number" -ForegroundColor Red
        return
    }

    $choice = [int]$choice

    if ($choice -ge $backups.Count) {
        Write-Host "Out of range" -ForegroundColor Red
        return
    }

    $selected = $backups[$choice]

    Write-Host ""
    Write-Host "RESTORING $($selected.Name)" -ForegroundColor Yellow

    Copy-Item $selected.FullName "src\parser_cobol.rs" -Force

    Write-Host "Parser restored"

    # BUILD TEST
    Write-Host ""
    Write-Host "=== VERIFYING BUILD ===" -ForegroundColor Cyan

    cargo build

    if ($LASTEXITCODE -ne 0) {
        Write-Host ""
        Write-Host "Restored parser still broken." -ForegroundColor Red
        return
    }

    Write-Host ""
    Write-Host "BASE BUILD SUCCESS" -ForegroundColor Green

    # LOAD CONTENT
    $path = "src\parser_cobol.rs"
    $content = Get-Content $path -Raw

    # FIND PERFORM ARM
    $start = $content.IndexOf('"perform" => {')

    if ($start -lt 0) {
        Write-Host "perform arm not found" -ForegroundColor Red
        return
    }

    $after = $content.Substring($start + 15)

    $regex = [regex]'(?m)^\s*"[a-zA-Z_]+"\s*=>\s*\{'

    $match = $regex.Match($after)

    if (!$match.Success) {
        Write-Host "next parser arm not found" -ForegroundColor Red
        return
    }

    $next = $start + 15 + $match.Index

    Write-Host ""
    Write-Host "PATCHING PERFORM UNTIL..."

    $newPerform = @'
"perform" => {

    if parts.len() >= 3 &&
       parts[1].to_lowercase() == "until" {

        let condition = parts[2..].join(" ");

        statements.push(
            Statement::PerformUntil {
                condition,
                body: vec![],
            }
        );

        continue;
    }

    anyhow::bail!("Invalid PERFORM: {}", line);
},

'@

    $newContent =
        $content.Substring(0, $start) +
        $newPerform +
        $content.Substring($next)

    Set-Content $path $newContent -Encoding UTF8

    Write-Host "Patch applied"

    # FINAL BUILD
    Write-Host ""
    Write-Host "=== FINAL BUILD ===" -ForegroundColor Cyan

    cargo build

    if ($LASTEXITCODE -ne 0) {

        Write-Host ""
        Write-Host "PATCH BUILD FAILED" -ForegroundColor Red
        return
    }

    Write-Host ""
    Write-Host "BUILD SUCCESS" -ForegroundColor Green

    # TESTS
    Write-Host ""
    Write-Host "=== RUNNING TESTS ===" -ForegroundColor Cyan

    cargo test --test semantic_validation

}
catch {

    Write-Host ""
    Write-Host "SCRIPT ERROR:" -ForegroundColor Red
    Write-Host $_
}

Write-Host ""
Write-Host "SCRIPT FINISHED"