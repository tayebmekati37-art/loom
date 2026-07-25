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
            let mut name = trimmed[5..].trim();

            name = name.trim_end_matches('.');

            let path = format!("copybooks/{}.cpy", name);

            if Path::new(&path).exists() {
                output.push_str(&fs::read_to_string(&path)?);

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
