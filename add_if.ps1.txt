$parserFile = "src/parser_cobol.rs"
$content = Get-Content $parserFile -Raw

# Replace the existing "if" arm with the new one (simple approach)
$oldIfArm = '"if" => \{\s*eprintln!\("IF not fully implemented, ignoring: {}", line\);\s*\}'
$newIfArm = @'
"if" => {
    let mut remaining = line;
    let after_if = remaining.trim_start_matches("if").trim_start();
    let then_pos = after_if.find(" then ").or_else(|| after_if.find(" then")).ok_or_else(|| anyhow::anyhow!("Missing THEN in IF statement: {}", line))?;
    let condition_str = after_if[..then_pos].trim();
    let after_then = after_if[then_pos + "then".len()..].trim_start();
    let (then_part, else_part) = if let Some(else_pos) = after_then.find(" else ") {
        (&after_then[..else_pos], Some(after_then[else_pos + "else".len()..].trim_start()))
    } else {
        (after_then, None)
    };
    let then_part = then_part.trim_end_matches(" end-if").trim_end();
    let condition = parse_condition_str(condition_str)?;
    let then_stmts = parse_statements_from_line(then_part)?;
    let else_stmts = if let Some(else_part) = else_part {
        let else_part = else_part.trim_end_matches(" end-if").trim_end();
        parse_statements_from_line(else_part)?
    } else {
        vec![]
    };
    statements.push(Statement::If {
        condition,
        then_branch: then_stmts,
        else_branch: if else_stmts.is_empty() { None } else { Some(else_stmts) },
    });
}
'@
$content = $content -replace $oldIfArm, $newIfArm

# Add helper functions if not already present
if ($content -notmatch "fn parse_condition_str") {
    $helpers = @'

fn parse_condition_str(s: &str) -> Result<Condition, anyhow::Error> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid condition: {}", s);
    }
    let left = parts[0].to_string();
    let operator = parts[1].to_string();
    let right = parts[2].parse::<i64>()?;
    Ok(Condition { left, operator, right })
}

fn parse_statements_from_line(line: &str) -> Result<Vec<Statement>, anyhow::Error> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(vec![]);
    }
    let stmt = match parts[0].to_lowercase().as_str() {
        "move" => {
            if parts.len() != 4 || parts[2].to_lowercase() != "to" {
                anyhow::bail!("Invalid MOVE statement: {}", line);
            }
            let source = if let Ok(num) = parts[1].parse::<i64>() {
                Source::Literal(num)
            } else {
                Source::Variable(parts[1].to_string())
            };
            let target = parts[3].to_string();
            Statement::Move { source, target }
        }
        "add" => {
            if parts.len() != 4 || parts[2].to_lowercase() != "to" {
                anyhow::bail!("Invalid ADD statement: {}", line);
            }
            let value = parts[1].parse::<i64>()?;
            let target = parts[3].to_string();
            Statement::Add { target, value }
        }
        "display" => {
            let lit_str = parts[1..].join(" ");
            let lit = if let Ok(num) = lit_str.parse::<i64>() {
                Literal::Int(num)
            } else {
                Literal::String(lit_str.trim_matches('\'').to_string())
            };
            Statement::Display { value: lit }
        }
        "perform" => {
            if parts.len() != 2 {
                anyhow::bail!("Invalid PERFORM statement: {}", line);
            }
            Statement::Perform { name: parts[1].to_string() }
        }
        _ => anyhow::bail!("Unsupported statement in IF block: {}", line),
    };
    Ok(vec![stmt])
}
'@
    $content = $content -replace '(\n\s*\}\s*)$', "$helpers`n$1"
}

$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText($parserFile, $content, $utf8NoBom)
Write-Host "Added IF support to parser_cobol.rs" -ForegroundColor Green