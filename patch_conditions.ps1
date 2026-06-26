Write-Host 'Patching condition parsing...'

$parser = Get-Content 'src\parser_cobol.rs' -Raw

if ($parser -notmatch 'fn parse_condition_expression') {

$conditionFns = @'

fn parse_condition_expression(cond: &str) -> crate::ir::Condition {

    let operators = vec![
        ">=",
        "<=",
        "!=",
        "=",
        ">",
        "<"
    ];

    for op in operators {

        if let Some(idx) = cond.find(op) {

            let left = cond[..idx].trim();
            let right = cond[idx + op.len()..].trim();

            return crate::ir::Condition {
                left: left.to_string(),
                operator: op.to_string(),
                right: right.to_string(),
            };
        }
    }

    crate::ir::Condition {
        left: cond.trim().to_string(),
        operator: "=".to_string(),
        right: "TRUE".to_string(),
    }
}

'@

$parser += "
" + $conditionFns
}

# replace naive IF parsing
$parser = $parser -replace 
'Condition\s*\{\s*left:\s*parts\[1\]\.to_string\(\),\s*operator:\s*parts\[2\]\.to_string\(\),\s*right:\s*parts\[3\]\.to_string\(\),\s*\}',
'parse_condition_expression(&line[2..])'

Set-Content 'src\parser_cobol.rs' $parser

Write-Host 'Patching interpreter condition evaluation...'

$interp = Get-Content 'src\interpreter.rs' -Raw

if ($interp -notmatch 'fn eval_condition_value') {

$evalHelper = @'

fn eval_condition_value(&self, value: &str) -> i64 {

    let value = value.trim();

    if let Ok(v) = value.parse::<i64>() {
        return v;
    }

    *self.vars.get(value).unwrap_or(&0)
}

'@

$interp = $interp -replace 
'impl Interpreter \{',
"impl Interpreter {
$evalHelper"
}

$interp = $interp -replace 
'let left = .*?;\s*let right = .*?;',
@'
let left = self.eval_condition_value(&condition.left);
                let right = self.eval_condition_value(&condition.right);
'@

Set-Content 'src\interpreter.rs' $interp

cargo check
cargo run
