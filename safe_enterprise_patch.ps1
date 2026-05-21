$ErrorActionPreference = "Stop"

Write-Host "SAFE ENTERPRISE PATCH" -ForegroundColor Cyan

$src = ".\src"

# ------------------------------------------------
# BACKUP
# ------------------------------------------------

$backup = "safe_backup_$(Get-Date -Format 'yyyyMMdd_HHmmss')"
New-Item -ItemType Directory -Path $backup | Out-Null

Copy-Item "$src\ir.rs" "$backup\ir.rs"
Copy-Item "$src\interpreter.rs" "$backup\interpreter.rs"

Write-Host "Backup created: $backup" -ForegroundColor Green

# ------------------------------------------------
# PATCH IR SAFELY
# ------------------------------------------------

$irFile = "$src\ir.rs"
$ir = Get-Content $irFile -Raw

if ($ir -notmatch "PerformUntil") {

$patch = @"

    PerformUntil {
        condition: Condition,
        body: Vec<Statement>,
    },

    Call {
        program: String,
    },

"@

$ir = $ir.Replace(
    "    Inspect { source: String, target: String, pattern: String },",
    "    Inspect { source: String, target: String, pattern: String },`r`n$patch"
)

Set-Content $irFile $ir -Encoding UTF8

Write-Host "IR upgraded safely" -ForegroundColor Green
}

# ------------------------------------------------
# PATCH INTERPRETER
# ------------------------------------------------

$interpFile = "$src\interpreter.rs"
$interp = Get-Content $interpFile -Raw

if ($interp -notmatch "PerformUntil") {

$matchPatch = @"

            crate::ir::Statement::PerformUntil { condition, body } => {
                while self.evaluate_condition(condition) {
                    self.execute_block(body);
                }
            }

            crate::ir::Statement::Call { program } => {
                println!("CALL {}", program);
            }

"@

$interp = $interp.Replace(
    "            crate::ir::Statement::Inspect { .. } => {}",
    "            crate::ir::Statement::Inspect { .. } => {},`r`n$matchPatch"
)

Set-Content $interpFile $interp -Encoding UTF8

Write-Host "Interpreter upgraded safely" -ForegroundColor Green
}

# ------------------------------------------------
# FORMAT + BUILD
# ------------------------------------------------

cargo fmt
cargo build

if ($LASTEXITCODE -eq 0) {

    Write-Host ""
    Write-Host "SAFE PATCH SUCCESSFUL" -ForegroundColor Green

} else {

    Write-Host ""
    Write-Host "BUILD FAILED" -ForegroundColor Red
    Write-Host "Restore from: $backup"
}