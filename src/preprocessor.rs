use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn preprocess(source: &str) -> Result<String> {
    let mut output = String::new();

    for line in source.lines() {
        let trimmed = line.trim();

        let upper = trimmed.to_uppercase();

        if upper.starts_with("COPY ") {
            let mut name = trimmed[5..].trim();

            name = name.trim_end_matches('.');

            let path = format!("copybooks/{}.cpy", name);

            if Path::new(&path).exists() {
                output.push_str(&fs::read_to_string(&path)?);

                output.push('\n');
            } else {
                output.push_str(line);

                output.push('\n');
            }
        } else {
            output.push_str(line);

            output.push('\n');
        }
    }

    Ok(output)
}
