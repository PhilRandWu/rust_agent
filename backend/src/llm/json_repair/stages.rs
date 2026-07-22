pub fn strip_bom(input: &str) -> String {
    if let Some(rest) = input.strip_prefix('\u{feff}') {
        rest.to_string()
    } else {
        input.to_string()
    }
}

pub fn strip_code_fence(input: &str) -> String {
    let trimmed = input.trim();
    if !trimmed.contains("```") {
        return input.to_string();
    }
    let after_open = match find_fence_start(trimmed) {
        Some(idx) => &trimmed[idx..],
        None => return input.to_string(),
    };
    let inner = match after_open.find("```") {
        Some(end) => &after_open[..end],
        None => after_open,
    };
    inner.trim().to_string()
}

fn find_fence_start(input: &str) -> Option<usize> {
    let start = input.find("```")?;
    let after_ticks = &input[start + 3..];
    let lang_end = after_ticks
        .find("\n")
        .map(|nl| nl + 1)
        .unwrap_or(after_ticks.len());
    Some(start + 3 + lang_end)
}

pub fn extract_first_json_object(input: &str) -> String {
    let bytes = input.as_bytes();
    let start = match bytes.iter().position(|&b| b == b'{' || b == b'[') {
        Some(idx) => idx,
        None => return input.to_string(),
    };
    let open = bytes[start];
    let close = if open == b'{' { b'}' } else { b']' };

    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;

    for (idx, &b) in bytes.iter().enumerate().skip(start) {
        if escape {
            escape = false;
            continue;
        }
        if in_string {
            match b {
                b'\\' => escape = true,
                b'"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            c if c == open => depth += 1,
            c if c == close => {
                depth -= 1;
                if depth == 0 {
                    return input[start..=idx].to_string();
                }
            }
            _ => {}
        }
    }
    input.to_string()
}

pub fn strip_trailing_commas(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escape = false;
    let chars: Vec<char> = input.chars().collect();

    for i in 0..chars.len() {
        let ch = chars[i];
        if escape {
            out.push(ch);
            escape = false;
            continue;
        }
        if in_string {
            if ch == '\\' {
                escape = true;
            }
            if ch == '"' {
                in_string = false;
            }
            out.push(ch);
            continue;
        }
        if ch == '"' {
            in_string = true;
            out.push(ch);
            continue;
        }
        if ch == ',' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                continue;
            }
        }
        out.push(ch);
    }
    out
}

pub fn balance_brackets(input: &str) -> String {
    let mut open_curly = 0i32;
    let mut open_square = 0i32;
    let mut in_string = false;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            escape = false;
            continue;
        }
        if in_string {
            match ch {
                '\\' => escape = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => open_curly += 1,
            '}' => open_curly -= 1,
            '[' => open_square += 1,
            ']' => open_square -= 1,
            _ => {}
        }
    }
    if in_string {
        return input.to_string();
    }
    if open_curly <= 0 && open_square <= 0 {
        return input.to_string();
    }
    let mut out = input.to_string();
    for _ in 0..open_square.max(0) {
        out.push(']');
    }
    for _ in 0..open_curly.max(0) {
        out.push('}');
    }
    out
}
