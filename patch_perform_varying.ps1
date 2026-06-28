Write-Host 'Adding PERFORM VARYING support...'

# ----------------------------
# PATCH IR
# ----------------------------

$ir = Get-Content 'src\ir.rs' -Raw

if ($ir -notmatch 'PerformVarying') {

    $ir = $ir -replace 
    'PerformUntil \{[\s\S]*?body: Vec<Statement>,\s*\},',
@'
PerformUntil {
        condition: Condition,
        body: Vec<Statement>,
    },

    PerformVarying {
        variable: String,
        from: Expression,
        by: Expression,
        until: Condition,
        body: Vec<Statement>,
    },
'@

    Set-Content 'src\ir.rs' $ir
}

# ----------------------------
# PATCH PARSER
# ----------------------------

$parser = Get-Content 'src\parser_cobol.rs' -Raw

if ($parser -notmatch 'PERFORM VARYING') {

$varyingBlock = @'

        if line.starts_with("PERFORM VARYING") {

            let text = line.replace("PERFORM VARYING", "").trim().to_string();

            let parts: Vec<&str> = text.split("UNTIL").collect();

            let left = parts[0].trim();
            let until_text = parts[1].trim();

            let left_parts: Vec<&str> = left.split_whitespace().collect();

            let variable = left_parts[0].to_string();

            let from_value = left_parts[2].parse::<i64>().unwrap_or(0);

            let by_value = left_parts[4].parse::<i64>().unwrap_or(1);

            *i += 1;

            let mut body = Vec::new();

            while *i < lines.len() {

                let inner = lines[*i].trim();

                if inner == "END-PERFORM" {
                    break;
                }

                body.push(parse_statement(inner)?);

                *i += 1;
            }

            return Ok(Statement::PerformVarying {
                variable,
                from: crate::ir::Expression::Literal(
                    crate::ir::Literal::Int(from_value)
                ),
                by: crate::ir::Expression::Literal(
                    crate::ir::Literal::Int(by_value)
                ),
                until: parse_condition_expression(until_text),
                body,
            });
        }

'@

    $parser = $parser -replace 
    'fn parse_statement\(line: &str\) -> Result<Statement> \{',
"fn parse_statement(line: &str) -> Result<Statement> {
$varyingBlock"

    Set-Content 'src\parser_cobol.rs' $parser
}

# ----------------------------
# PATCH INTERPRETER
# ----------------------------

$interp = Get-Content 'src\interpreter.rs' -Raw

if ($interp -notmatch 'PerformVarying') {

$varyingExec = @'

            crate::ir::Statement::PerformVarying {
                variable,
                from,
                by,
                until,
                body,
            } => {

                let mut current =
                    self.evaluate_expression(from);

                let step =
                    self.evaluate_expression(by);

                self.vars.insert(variable.clone(), current);

                loop {

                    let left =
                        self.eval_condition_value(&until.left);

                    let right =
                        self.eval_condition_value(&until.right);

                    let done = match until.operator.as_str() {

                        "=" => left == right,
                        "!=" => left != right,
                        ">" => left > right,
                        "<" => left < right,
                        ">=" => left >= right,
                        "<=" => left <= right,

                        _ => false,
                    };

                    if done {
                        break;
                    }

                    for stmt in body {
                        self.execute_statement(stmt)?;
                    }

                    current += step;

                    self.vars.insert(variable.clone(), current);
                }
            }

'@

    $interp = $interp -replace 
    '(crate::ir::Statement::PerformUntil[\s\S]*?\n\s*\})',
"$1
$varyingExec"

    Set-Content 'src\interpreter.rs' $interp
}

cargo check
cargo build
