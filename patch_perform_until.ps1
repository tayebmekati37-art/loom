Write-Host 'Adding PERFORM UNTIL support...'

$ir = Get-Content 'src\ir.rs' -Raw

if ($ir -notmatch 'PerformUntil') {

    $ir = $ir -replace 
    'Perform \{[^}]*\},',
@'
Perform {
        paragraph: String,
    },

    PerformUntil {
        condition: Condition,
        body: Vec<Statement>,
    },
'@

    Set-Content 'src\ir.rs' $ir
}

$parser = Get-Content 'src\parser_cobol.rs' -Raw

if ($parser -notmatch 'PERFORM UNTIL') {

$performUntil = @'

        if line.starts_with("PERFORM UNTIL") {

            let cond_text = line
                .replace("PERFORM UNTIL", "")
                .trim()
                .to_string();

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

            return Ok(Statement::PerformUntil {
                condition: parse_condition_expression(&cond_text),
                body,
            });
        }

'@

    $parser = $parser -replace 
    'fn parse_statement\(line: &str\) -> Result<Statement> \{',
"fn parse_statement(line: &str) -> Result<Statement> {
$performUntil"

    Set-Content 'src\parser_cobol.rs' $parser
}

$interp = Get-Content 'src\interpreter.rs' -Raw

if ($interp -notmatch 'PerformUntil') {

$loopImpl = @'

            crate::ir::Statement::PerformUntil { condition, body } => {

                loop {

                    let left =
                        self.eval_condition_value(&condition.left);

                    let right =
                        self.eval_condition_value(&condition.right);

                    let done = match condition.operator.as_str() {

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
                }
            }

'@

    $interp = $interp -replace 
    '(crate::ir::Statement::Perform\s*\{[^}]*\}\s*=>\s*\{[^}]*\})',
"$1
$loopImpl"

    Set-Content 'src\interpreter.rs' $interp
}

cargo check
cargo build
