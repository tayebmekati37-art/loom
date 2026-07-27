use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn preprocess(source: &str) -> Result<String> {
    let mut output = String::new();
    let mut replacements: Vec<(String, String)> = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();

        let upper = trimmed.to_uppercase();

        if upper.starts_with("REPLACE ") {
            let text = trimmed
                .trim_start_matches("REPLACE")
                .trim()
                .trim_end_matches('.');

            if let Some((old, new)) = text.split_once(" BY ") {
                let old = old.trim().trim_matches('=').to_string();
                let new = new.trim().trim_matches('=').to_string();

                replacements.push((old, new));
            }

            continue;
        }

        if upper.starts_with("COPY ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();

            let name = parts[1].trim_end_matches('.');

            let mut local_text = String::new();

            let path = format!("copybooks/{}.cpy", name);

            if Path::new(&path).exists() {
                local_text = fs::read_to_string(&path)?;

                if let Some(pos) = upper.find("REPLACING") {
                    let clause = &trimmed[pos + "REPLACING".len()..];

                    if let Some((old, new)) = clause.split_once("BY") {
                        let old = old.trim().trim_matches('=');

                        let new = new.trim().trim_end_matches('.').trim_matches('=');

                        local_text = local_text.replace(old, new);
                    }
                }

                for (old, new) in &replacements {
                    local_text = local_text.replace(old, new);
                }

                output.push_str(&local_text);

                output.push('\n');
            } else {
                let mut expanded = line.to_string();

                for (old, new) in &replacements {
                    expanded = expanded.replace(old, new);
                }

                output.push_str(&expanded);

                output.push('\n');
            }
        } else {
            let mut expanded = line.to_string();

            for (old, new) in &replacements {
                expanded = expanded.replace(old, new);
            }

            output.push_str(&expanded);

            output.push('\n');
        }
    }

    Ok(output)
}
