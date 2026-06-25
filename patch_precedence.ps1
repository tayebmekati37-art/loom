Write-Host 'Adding precedence parser...'

$parser = Get-Content 'src\parser_cobol.rs' -Raw

# remove old parse_expression
$parser = [regex]::Replace(
    $parser,
    '(?s)fn parse_expression\(expr: &str\).*?\n\}',
    ''
)

$newParser = @'

fn parse_expression(expr: &str) -> crate::ir::Expression {
    parse_add_sub(expr.trim())
}

fn parse_add_sub(expr: &str) -> crate::ir::Expression {

    let mut depth = 0;

    for (i, ch) in expr.char_indices().rev() {

        match ch {
            ')' => depth += 1,
            '(' => depth -= 1,

            '+' | '-' if depth == 0 => {

                let left = expr[..i].trim();
                let right = expr[i + 1..].trim();

                return crate::ir::Expression::Binary {
                    left: Box::new(parse_add_sub(left)),
                    operator: ch.to_string(),
                    right: Box::new(parse_mul_div(right)),
                };
            }

            _ => {}
        }
    }

    parse_mul_div(expr)
}

fn parse_mul_div(expr: &str) -> crate::ir::Expression {

    let mut depth = 0;

    for (i, ch) in expr.char_indices().rev() {

        match ch {
            ')' => depth += 1,
            '(' => depth -= 1,

            '*' | '/' if depth == 0 => {

                let left = expr[..i].trim();
                let right = expr[i + 1..].trim();

                return crate::ir::Expression::Binary {
                    left: Box::new(parse_mul_div(left)),
                    operator: ch.to_string(),
                    right: Box::new(parse_primary(right)),
                };
            }

            _ => {}
        }
    }

    parse_primary(expr)
}

fn parse_primary(expr: &str) -> crate::ir::Expression {

    let expr = expr.trim();

    if expr.starts_with("(") && expr.ends_with(")") {
        return parse_expression(&expr[1..expr.len()-1]);
    }

    if let Ok(v) = expr.parse::<i64>() {
        return crate::ir::Expression::Literal(
            crate::ir::Literal::Int(v)
        );
    }

    crate::ir::Expression::Variable(expr.to_string())
}

'@

$parser += "
" + $newParser

Set-Content 'src\parser_cobol.rs' $parser

Write-Host 'Running cargo check...'
cargo check

Write-Host 'Running cargo test...'
cargo test
