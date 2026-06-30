
# =========================================
# Loom Compiler Architecture Bootstrap
# =========================================

Write-Host "Creating compiler architecture..." -ForegroundColor Cyan

# -----------------------------
# Create source files
# -----------------------------

$files = @(
    "src\ast.rs",
    "src\lowering.rs"
)

foreach ($file in $files) {
    if (!(Test-Path $file)) {
        New-Item -ItemType File -Path $file | Out-Null
        Write-Host "Created $file"
    }
}

# -----------------------------
# AST
# -----------------------------

$ast = @'
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstStatement {

    Move {
        source: String,
        target: String,
    },

    Add {
        value: String,
        target: String,
    },

    Display {
        value: String,
    },

    If {
        condition: AstCondition,
        then_branch: Vec<AstStatement>,
        else_branch: Option<Vec<AstStatement>>,
    },

    PerformUntil {
        condition: AstCondition,
        body: Vec<AstStatement>,
    },

    PerformVarying {
        variable: String,
        from: String,
        by: String,
        until: AstCondition,
        body: Vec<AstStatement>,
    },

    Evaluate {
        value: String,
        whens: Vec<AstWhen>,
    },

    String {
        sources: Vec<String>,
        target: String,
    },

    Unstring {
        source: String,
        delimiter: String,
        targets: Vec<String>,
    },

    Inspect {
        source: String,
        replacing_from: String,
        replacing_to: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstCondition {
    pub left: String,
    pub operator: String,
    pub right: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstWhen {
    pub condition: String,
    pub body: Vec<AstStatement>,
}
'@

Set-Content -Path "src\ast.rs" -Value $ast

# -----------------------------
# LOWERING
# -----------------------------

$lowering = @'
use crate::ast::*;
use crate::ir::*;

pub fn lower_program(
    ast: &[AstStatement],
) -> Vec<Statement> {

    ast.iter()
        .map(lower_statement)
        .collect()
}

pub fn lower_statement(
    stmt: &AstStatement,
) -> Statement {

    match stmt {

        AstStatement::Display { value } => {
            Statement::Display {
                value: Expression::Variable(value.clone()),
            }
        }

        AstStatement::Move {
            source,
            target,
        } => {

            Statement::Assign {
                target: target.clone(),
                value: Expression::Variable(source.clone()),
            }
        }

        AstStatement::Add {
            value,
            target,
        } => {

            Statement::Assign {
                target: target.clone(),

                value: Expression::Binary {
                    left: Box::new(
                        Expression::Variable(target.clone())
                    ),

                    operator: "+".to_string(),

                    right: Box::new(
                        Expression::Variable(value.clone())
                    ),
                },
            }
        }

        _ => Statement::NoOp,
    }
}
'@

Set-Content -Path "src\lowering.rs" -Value $lowering

# -----------------------------
# Update main.rs modules
# -----------------------------

$mainPath = "src\main.rs"

if (Test-Path $mainPath) {

    $content = Get-Content $mainPath -Raw

    if ($content -notmatch "mod ast;") {
        $content = "mod ast;`r`n" + $content
    }

    if ($content -notmatch "mod lowering;") {
        $content = "mod lowering;`r`n" + $content
    }

    Set-Content $mainPath $content

    Write-Host "Updated main.rs"
}

# -----------------------------
# Git checkpoint
# -----------------------------

git add .

git commit -m "bootstrap AST and lowering architecture"

Write-Host ""
Write-Host "Architecture bootstrap complete." -ForegroundColor Green
Write-Host ""
Write-Host "Next:"
Write-Host "1. cargo check"
Write-Host "2. migrate parser -> AST"
Write-Host "3. migrate interpreter -> IR"

