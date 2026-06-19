use crate::ir::*;
use anyhow::Result;

pub fn parse_program(input: &str) -> Result<Vec<Statement>> {
    let mut statements = Vec::new();
    let mut variables: Vec<crate::ir::VariableDefinition> = Vec::new();

    let lines: Vec<&str> = input.lines().collect();
let mut i = 0;

while i < lines.len() {
    let raw_line = lines[i];
        let line = raw_line.replace('\u{feff}', "").trim().to_string();

        let line = line.as_str();
        // Skip empty lines
        if line.is_empty() {
            i += 1;
         continue;
        }

        // Skip comments
        if line.starts_with('*') {
            i += 1;
           continue;
        }

        // Normalize for matching
        let upper = line.to_uppercase();

        // Skip COBOL divisions / metadata
        if upper.starts_with("IDENTIFICATION DIVISION")
            || upper.starts_with("ENVIRONMENT DIVISION")
            || upper.starts_with("DATA DIVISION")
            || upper.starts_with("PROCEDURE DIVISION")
            || upper.starts_with("WORKING-STORAGE SECTION")
            || upper.starts_with("FILE SECTION")
            || upper.starts_with("LINKAGE SECTION")
            || upper.starts_with("PROGRAM-ID")
            || upper.starts_with("AUTHOR")
            || upper.starts_with("DATE-WRITTEN")
        {
            i += 1;
         continue;
        }

        if parse_variable_definition(line).is_some() {
            i += 1;
          continue;
        }
        if upper.starts_with("IF ") {

    let mut body = Vec::new();

    i += 1;

    while i < lines.len() {

        let body_line = lines[i].trim();

        if body_line.eq_ignore_ascii_case("END-IF")
            || body_line.eq_ignore_ascii_case("END-IF.")
        {
            break;
        }

        let stmt = parse_statement(body_line)?;
        body.push(stmt);

        i += 1;
    }

    let if_stmt = parse_statement(line)?;

    match if_stmt {
        Statement::If {
            condition,
            else_branch,
            ..
        } => {
            statements.push(
                Statement::If {
                    condition,
                    then_branch: body,
                    else_branch,
                }
            );
        }
        _ => statements.push(if_stmt),
    }

} else {

    let stmt = parse_statement(line)?;
    statements.push(stmt);

}
i += 1;
}

    Ok(statements)
}

fn parse_block(
lines: &Vec<&str>,
i: &mut usize,
terminators: &[&str],
) -> Result<Vec<Statement>> {


let mut statements = Vec::new();

while *i < lines.len() {

    let raw = lines[*i];
    let line = raw.trim();

    if line.is_empty() {
        *i += 1;
        continue;
    }

    let upper = line.to_uppercase();

    if terminators.iter().any(|t| upper == *t) {
        break;
    }

    // =========================
    // IF BLOCK
    // =========================

    if upper.starts_with("IF ") {

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 4 {
            anyhow::bail!("Invalid IF");
        }

        let condition = Condition {
            left: parts[1].to_string(),
            operator: parts[2].to_string(),
            right: parts[3].to_string(),
        };

        *i += 1;

        let then_branch =
            parse_block(lines, i, &["ELSE", "END-IF"])?;

        let mut else_branch = None;

        if *i < lines.len()
            && lines[*i].trim().eq_ignore_ascii_case("ELSE")
        {
            *i += 1;

            else_branch =
                Some(parse_block(lines, i, &["END-IF"])?);
        }

        if *i < lines.len()
            && lines[*i].trim().eq_ignore_ascii_case("END-IF")
        {
            *i += 1;
        }

        statements.push(Statement::If {
            condition,
            then_branch,
            else_branch,
        });

        continue;
    }

    // =========================
    // PERFORM BLOCK
    // =========================

    if upper.starts_with("PERFORM UNTIL ") {

        let parts: Vec<&str> = line.split_whitespace().collect();

        let condition = Condition {
            left: parts[2].to_string(),
            operator: ">".to_string(),
            right: "0".to_string(),
        };

        *i += 1;

        let body =
            parse_block(lines, i, &["END-PERFORM"])?;

        if *i < lines.len()
            && lines[*i].trim().eq_ignore_ascii_case("END-PERFORM")
        {
            *i += 1;
        }

        statements.push(Statement::PerformUntil {
            condition,
            body,
        });

        continue;
    }

    // =========================
    // NORMAL STATEMENT
    // =========================

    let stmt = parse_statement(line)?;
    statements.push(stmt);

    *i += 1;
}

Ok(statements)


}

fn parse_variable_definition(line: &str) -> Option<VariableDefinition> {
    let clean = line.replace(".", "");
    let parts: Vec<&str> = clean.split_whitespace().collect();

    if parts.len() < 4 {
        return None;
    }

    if parts[0] != "01" {
        return None;
    }

    if !parts[2].eq_ignore_ascii_case("PIC") {
        return None;
    }

    let name = parts[1].to_string();

    let pic_text = parts[3].to_uppercase();

    let pic = if pic_text.contains('V') {
        Some(PicType::Decimal)
    } else if pic_text.contains('9') {
        Some(PicType::Numeric)
    } else {
        Some(PicType::Alpha)
    };

    let comp_type = if clean.to_uppercase().contains("COMP-3") {
        Some(CompType::Comp3)
    } else if clean.to_uppercase().contains("COMP") {
        Some(CompType::Comp)
    } else {
        None
    };

    Some(VariableDefinition {
        name,
        pic,
        occurs: None,
        redefines: None,
        initial_value: None,
        comp_type,
    })
}
fn parse_statement(line: &str) -> Result<Statement> {
    let upper = line.trim().to_uppercase();

    if upper.starts_with("END-") {
        return Ok(Statement::NoOp);
    }

    // Ignore PIC declarations
    if line.to_uppercase().contains("PIC ") {
        return Ok(Statement::NoOp);
    }

    let clean = line.replace('.', "");
    let parts: Vec<&str> = clean.split_whitespace().collect();

    if parts.is_empty() {
        anyhow::bail!("Empty statement");
    }

    match parts[0].to_lowercase().as_str() {
        "display" => {
            if parts.len() < 2 {
                anyhow::bail!("Invalid DISPLAY");
            }

            Ok(Statement::Display {
                value: Literal::String(parts[1..].join(" ")),
            })
        }

        "move" => {
            if parts.len() < 4 {
                anyhow::bail!("Invalid MOVE");
            }

            Ok(Statement::Move {
                source: Source::Variable(parts[1].to_string()),
                target: parts[3].to_string(),
            })
        }

        "add" => {
            if parts.len() < 4 {
                anyhow::bail!("Invalid ADD");
            }

            Ok(Statement::Add {
                value: parts[1].parse::<i64>().unwrap_or(0),
                target: parts[3].to_string(),
            })
        }

        "perform" => {
            if parts.len() >= 3 && parts[1].eq_ignore_ascii_case("until") {
                let cond = Condition {
                 left: parts[2].to_string(),
                operator: ">".to_string(),
                right: "0".to_string(),
                };

                Ok(Statement::PerformUntil {
                    condition: cond,
                    body: Vec::new(),
                })
            } else if parts.len() >= 2 {
                Ok(Statement::Perform {
                  name: Some(parts[1].to_string()),
                  body: Vec::new(),
                })
            } else {
                anyhow::bail!("Invalid PERFORM");
            }
        }

        "call" => {
            if parts.len() < 2 {
                anyhow::bail!("Invalid CALL");
            }

            Ok(Statement::Call {
                program: parts[1].replace('"', ""),
                using_args: Vec::new(),
            })
        }

        "if" => {
            if parts.len() < 4 {
                anyhow::bail!("Invalid IF");
            }

            let left = parts[1].to_string();
            let operator = parts[2].to_string();

            let right = parts[3].to_string();

            let cond = Condition {
                left,
                operator,
                right,
            };

            Ok(Statement::If {
                condition: cond,
                then_branch: Vec::new(),
                else_branch: None,
            })
        }

        "stop" => {
            if parts.len() >= 2 && parts[1].eq_ignore_ascii_case("run") {
                Ok(Statement::NoOp)
            } else {
                anyhow::bail!("Invalid STOP statement");
            }
        }
        "compute" => {
            if parts.len() < 4 {
                anyhow::bail!("Invalid COMPUTE");
            }

            let target = parts[1].to_string();

            let expr = parts[3..].join(" ");

            Ok(Statement::Compute { target, expr })
        }
        "end-perform" => Ok(Statement::NoOp),
        _ => {
            anyhow::bail!("Unknown statement: {}", line)
        }
    }
}







