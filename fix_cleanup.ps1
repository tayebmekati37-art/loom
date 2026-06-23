# =========================
# CLEAN DUPLICATE EXPRESSION ENUMS
# =========================

$ir = Get-Content src\ir.rs -Raw

$matches = [regex]::Matches(
    $ir,
    '(?s)#\[derive\(Debug, Clone, Serialize, Deserialize\)\]\s*pub enum Expression\s*\{.*?\n\}'
)

if ($matches.Count -gt 1) {
    $first = $matches[0].Value

    $ir = [regex]::Replace(
        $ir,
        '(?s)#\[derive\(Debug, Clone, Serialize, Deserialize\)\]\s*pub enum Expression\s*\{.*?\n\}',
        '',
        1
    )

    $ir = $first + "`r`n`r`n" + $ir
}

Set-Content src\ir.rs $ir

$interp = Get-Content src\interpreter.rs -Raw

$interp = $interp.Replace(
    'Vec[crate::ir::Statement](crate::ir::Statement)',
    'Vec<crate::ir::Statement>'
)

$interp = $interp.Replace(
    'para.body.clone()',
    'para.statements.clone()'
)

$interp = $interp.Replace(
@"
Self {
            vars: HashMap::new(),
            functions: HashMap::new(),
        }
"@,
@"
Self {
            vars: HashMap::new(),
            functions: HashMap::new(),
            paragraphs: HashMap::new(),
        }
"@
)

Set-Content src\interpreter.rs $interp

Write-Host ""
Write-Host "cleanup complete"
