Write-Host 'Patching parser_cobol.rs...'

$parser = Get-Content 'src\parser_cobol.rs' -Raw

if ($parser -notmatch 'fn parse_expression') {

$helper = @'

fn parse_expression(expr: &str) -> crate::ir::Expression {
    let expr = expr.trim();

    if let Ok(v) = expr.parse::<i64>() {
        return crate::ir::Expression::Literal(
            crate::ir::Literal::Int(v)
        );
    }

    for op in ["+", "-", "*", "/"] {
        if let Some(idx) = expr.find(op) {
            let left = expr[..idx].trim();
            let right = expr[idx + 1..].trim();

            return crate::ir::Expression::Binary {
                left: Box::new(parse_expression(left)),
                operator: op.to_string(),
                right: Box::new(parse_expression(right)),
            };
        }
    }

    crate::ir::Expression::Variable(expr.to_string())
}

'@

$parser += "
" + $helper
}

Set-Content 'src\parser_cobol.rs' $parser

Write-Host 'Patching interpreter.rs...'

$interp = Get-Content 'src\interpreter.rs' -Raw

if ($interp -notmatch 'fn evaluate_expression') {

$eval = @'

fn evaluate_expression(&self, expr: &crate::ir::Expression) -> i64 {
    match expr {

        crate::ir::Expression::Literal(lit) => {
            match lit {
                crate::ir::Literal::Int(v) => *v,
                _ => 0,
            }
        }

        crate::ir::Expression::Variable(name) => {
            *self.vars.get(name).unwrap_or(&0)
        }

        crate::ir::Expression::Binary {
            left,
            operator,
            right,
        } => {

            let l = self.evaluate_expression(left);
            let r = self.evaluate_expression(right);

            match operator.as_str() {
                "+" => l + r,
                "-" => l - r,
                "*" => l * r,
                "/" => {
                    if r == 0 { 0 } else { l / r }
                }
                _ => 0,
            }
        }
    }
}

'@

$interp = $interp -replace '\}\s*$', "$eval
}"
}

$interp = $interp -replace 
'let value = self\.evaluate_expression\(expr\);\s*self\.variables\.insert',
'let value = self.evaluate_expression(expr);
                 self.vars.insert'

Set-Content 'src\interpreter.rs' $interp

cargo check
cargo test
