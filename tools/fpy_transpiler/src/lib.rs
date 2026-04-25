#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    Root,
    Generic,
    Struct,
    Enum,
    Match,
    ClosureExpr,
}

const FORBIDDEN_CONSTRUCTS: [&str; 6] = ["try", "except", "lambda", "async", "yield", "with"];

pub fn transpile_source(source: &str, source_name: &str) -> Result<String, String> {
    let mut output = Vec::new();
    output.push("// AUTO-GENERATED FILE -- DO NOT EDIT".to_string());
    output.push(format!("// Source: {}", source_name));

    let mut stack: Vec<(usize, BlockKind)> = vec![(0, BlockKind::Root)];
    let mut pending_block: Option<(BlockKind, usize)> = None;

    for (line_index, raw_line) in source.lines().enumerate() {
        let line_no = line_index + 1;
        ensure_no_tab_indentation(raw_line, line_no)?;

        let indent = count_leading_spaces(raw_line);
        let trimmed = raw_line[indent..].trim_end();
        let (code_part, comment_part) = split_code_and_comment(trimmed);
        let code = code_part.trim_end();
        let is_blank = trimmed.is_empty();
        let is_comment_only = code.is_empty() && comment_part.is_some();
        let is_significant = !code.is_empty();

        if is_significant {
            if let Some((block_kind, header_line)) = pending_block.take() {
                let current_indent = stack.last().map(|(n, _)| *n).unwrap_or(0);
                if indent <= current_indent {
                    return Err(format!(
                        "Expected an indented block after line {}",
                        header_line
                    ));
                }
                stack.push((indent, block_kind));
            }

            while indent < stack.last().map(|(n, _)| *n).unwrap_or(0) {
                let (_, closed_kind) = stack.pop().expect("stack must contain block");
                let close_indent = stack.last().map(|(n, _)| *n).unwrap_or(0);
                let close_token = if matches!(closed_kind, BlockKind::ClosureExpr) {
                    "};"
                } else {
                    "}"
                };
                output.push(format!("{}{}", " ".repeat(close_indent), close_token));
            }

            let current_indent = stack.last().map(|(n, _)| *n).unwrap_or(0);
            if indent > current_indent {
                return Err(format!("Unexpected indentation at line {}", line_no));
            }
        }

        if is_blank {
            output.push(String::new());
            continue;
        }

        if is_comment_only {
            output.push(format!(
                "{}{}",
                " ".repeat(indent),
                as_slash_comment(comment_part.unwrap_or_default())
            ));
            continue;
        }

        check_forbidden_constructs(code, line_no)?;

        if code.ends_with(':') {
            let (header, block_kind) = transform_header(code, line_no)?;
            let mut rendered = format!("{}{}", " ".repeat(indent), header);
            if let Some(comment) = comment_part {
                rendered.push(' ');
                rendered.push_str(&as_slash_comment(comment));
            }
            output.push(rendered);
            pending_block = Some((block_kind, line_no));
            continue;
        }

        let parent_kind = stack
            .last()
            .map(|(_, kind)| *kind)
            .unwrap_or(BlockKind::Root);
        let mut rendered_code = code.to_string();
        if should_add_semicolon(&rendered_code, parent_kind) {
            rendered_code.push(';');
        }

        let mut rendered = format!("{}{}", " ".repeat(indent), rendered_code);
        if let Some(comment) = comment_part {
            rendered.push(' ');
            rendered.push_str(&as_slash_comment(comment));
        }
        output.push(rendered);
    }

    if let Some((_, header_line)) = pending_block {
        return Err(format!(
            "Expected an indented block after line {}",
            header_line
        ));
    }

    while stack.len() > 1 {
        let (_, closed_kind) = stack.pop().expect("stack must contain block");
        let close_indent = stack.last().map(|(n, _)| *n).unwrap_or(0);
        let close_token = if matches!(closed_kind, BlockKind::ClosureExpr) {
            "};"
        } else {
            "}"
        };
        output.push(format!("{}{}", " ".repeat(close_indent), close_token));
    }

    Ok(format!("{}\n", output.join("\n")))
}

fn ensure_no_tab_indentation(line: &str, line_no: usize) -> Result<(), String> {
    for ch in line.chars() {
        if ch == ' ' {
            continue;
        }
        if ch == '\t' {
            return Err(format!(
                "Tabs are not supported for indentation (line {}). Use spaces only.",
                line_no
            ));
        }
        break;
    }
    Ok(())
}

fn count_leading_spaces(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ').count()
}

fn split_code_and_comment(line: &str) -> (&str, Option<&str>) {
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    for (idx, ch) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        if ch == '\\' && (in_single || in_double) {
            escaped = true;
            continue;
        }

        if ch == '\'' && !in_double {
            in_single = !in_single;
            continue;
        }

        if ch == '"' && !in_single {
            in_double = !in_double;
            continue;
        }

        if ch == '#' && !in_single && !in_double {
            let code = line[..idx].trim_end();
            let comment = line[idx + 1..].trim_start();
            return (code, Some(comment));
        }
    }

    (line.trim_end(), None)
}

fn as_slash_comment(comment: &str) -> String {
    if comment.is_empty() {
        "//".to_string()
    } else {
        format!("// {}", comment)
    }
}

fn check_forbidden_constructs(code: &str, line_no: usize) -> Result<(), String> {
    for token in FORBIDDEN_CONSTRUCTS {
        if contains_word(code, token) {
            return Err(format!(
                "Forbidden Python construct '{}' at line {}",
                token, line_no
            ));
        }
    }
    Ok(())
}

fn contains_word(haystack: &str, needle: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        let abs = start + pos;
        let prev = haystack[..abs].chars().next_back();
        let next = haystack[abs + needle.len()..].chars().next();

        let prev_ok = prev.map(is_word_boundary).unwrap_or(true);
        let next_ok = next.map(is_word_boundary).unwrap_or(true);
        if prev_ok && next_ok {
            return true;
        }

        start = abs + needle.len();
    }
    false
}

fn is_word_boundary(ch: char) -> bool {
    !ch.is_ascii_alphanumeric() && ch != '_'
}

fn transform_header(code: &str, line_no: usize) -> Result<(String, BlockKind), String> {
    let mut header = code
        .strip_suffix(':')
        .ok_or_else(|| format!("Invalid header syntax at line {}", line_no))?
        .trim_end()
        .to_string();

    if header.starts_with("def ") {
        header = format!("func {}", &header[4..]);
    } else if header.starts_with("elif ") {
        header = format!("else if {}", &header[5..]);
    }

    if header.is_empty() {
        return Err(format!("Invalid empty header at line {}", line_no));
    }

    let block_kind = classify_block_kind(&header);
    Ok((format!("{} {{", header), block_kind))
}

fn classify_block_kind(header: &str) -> BlockKind {
    if header.starts_with("struct ") {
        BlockKind::Struct
    } else if header.starts_with("enum ") {
        BlockKind::Enum
    } else if header.starts_with("match ") {
        BlockKind::Match
    } else if header.contains('|') && (header.contains("= |") || header.starts_with("return |")) {
        BlockKind::ClosureExpr
    } else {
        BlockKind::Generic
    }
}

fn should_add_semicolon(code: &str, parent_kind: BlockKind) -> bool {
    let trimmed = code.trim_end();
    if trimmed.is_empty()
        || trimmed.ends_with(';')
        || trimmed.ends_with(',')
        || trimmed.ends_with('{')
    {
        return false;
    }

    if trimmed == "}" {
        return false;
    }

    if matches!(parent_kind, BlockKind::Struct | BlockKind::Enum) {
        return false;
    }

    if matches!(parent_kind, BlockKind::Match) && trimmed.contains("=>") {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::transpile_source;

    #[test]
    fn transpiles_basic_function_and_control_flow() {
        let source = r#"def main():
    let x = 1
    if x == 1:
        println("ok")
    else:
        println("no")
"#;

        let out = transpile_source(source, "main.fpy").expect("transpile must succeed");
        assert!(out.contains("func main() {"));
        assert!(out.contains("let x = 1;"));
        assert!(out.contains("if x == 1 {"));
        assert!(out.contains("else {"));
        assert!(out.contains("println(\"ok\");"));
    }

    #[test]
    fn keeps_struct_and_enum_members_without_semicolons() {
        let source = r#"struct Point:
    x: i64
    y: i64

enum State:
    Ready
    Failed(i64)
"#;

        let out = transpile_source(source, "types.fpy").expect("transpile must succeed");
        assert!(out.contains("struct Point {"));
        assert!(out.contains("    x: i64"));
        assert!(!out.contains("x: i64;"));
        assert!(out.contains("enum State {"));
        assert!(out.contains("    Ready"));
        assert!(!out.contains("Ready;"));
    }

    #[test]
    fn rejects_forbidden_python_constructs() {
        let source = r#"def main():
    try:
        println("no")
"#;

        let err = transpile_source(source, "bad.fpy").expect_err("expected forbidden construct");
        assert!(err.contains("Forbidden Python construct 'try'"));
    }

    #[test]
    fn rejects_tab_indentation() {
        let source = "def main():\n\tprintln(\"bad\")\n";
        let err = transpile_source(source, "tabs.fpy").expect_err("expected tab error");
        assert!(err.contains("Tabs are not supported"));
    }

    #[test]
    fn adds_semicolon_for_struct_literal_and_closure_lines_ending_with_brace() {
        let source = r#"def main():
    let p = Point { x: 10, y: 20 }
    let adder = |x: i64| -> i64:
        return x + 1
    return Point { x: 1, y: adder(2) }
"#;

        let out = transpile_source(source, "brace_semicolon.fpy").expect("transpile must succeed");
        assert!(out.contains("let p = Point { x: 10, y: 20 };"));
        assert!(out.contains("let adder = |x: i64| -> i64 {"));
        assert!(out.contains("    };"));
        assert!(out.contains("return Point { x: 1, y: adder(2) };"));
    }
}
