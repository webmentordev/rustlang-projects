fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("City-country-latitude-longitude-query.pdf").unwrap();
    let out = pdf_extract::extract_text_from_mem(&bytes).unwrap();

    let lines: Vec<&str> = out.lines().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty()
            || line.contains("www.jinyi-solar.com")
            || line.contains("City Province/State Country")
        {
            i += 1;
            continue;
        }
        if i + 1 < lines.len() {
            let next_line = lines[i + 1].trim();
            if !next_line.is_empty()
                && next_line
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_numeric() || c == '-')
            {
                result.push_str(line);
                result.push(' ');
                result.push_str(next_line);
                result.push('\n');
                i += 2;
                continue;
            }
        }
        result.push_str(line);
        result.push('\n');
        i += 1;
    }

    println!("{}", result);
    Ok(())
}
