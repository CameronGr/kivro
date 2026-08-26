#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Lang {
    Rust,
    Bash,
    Toml,
    Json,
    TypeScript,
    Css,
    #[default]
    Plain,
}

impl Lang {
    /// Resolve a language tag, matching the aliases the React docs accepted.
    pub fn from_tag(tag: &str) -> Self {
        match tag.trim().to_ascii_lowercase().as_str() {
            "rust" | "rs" => Lang::Rust,
            "bash" | "sh" | "shell" | "zsh" | "console" => Lang::Bash,
            "toml" | "ini" | "cfg" => Lang::Toml,
            "json" | "jsonc" => Lang::Json,
            "ts" | "typescript" | "tsx" | "js" | "javascript" | "jsx" => Lang::TypeScript,
            "css" | "scss" | "sass" | "postcss" => Lang::Css,
            _ => Lang::Plain,
        }
    }

    /// The label shown in a code block header when no title is given.
    pub const fn label(self) -> &'static str {
        match self {
            Lang::Rust => "rust",
            Lang::Bash => "bash",
            Lang::Toml => "toml",
            Lang::Json => "json",
            Lang::TypeScript => "typescript",
            Lang::Css => "css",
            Lang::Plain => "text",
        }
    }

    const fn spec(self) -> Spec {
        match self {
            Lang::Rust => Spec {
                line_comment: "//",
                block_comment: Some(("/*", "*/")),
                double_quote: true,
                single_quote: true,
                backtick: false,
                keywords: RUST_KEYWORDS,
                constants: RUST_CONSTANTS,
                lifetimes: true,
                attributes: true,
                shell_vars: false,
                headers: false,
                upper_is_type: true,
                macro_bang: true,
                key_value: false,
                flags: false,
                dashed_idents: false,
                numbers: true,
                calls: true,
            },
            Lang::Bash => Spec {
                line_comment: "#",
                block_comment: None,
                double_quote: true,
                single_quote: true,
                backtick: true,
                keywords: BASH_KEYWORDS,
                constants: &[],
                lifetimes: false,
                attributes: false,
                shell_vars: true,
                headers: false,
                upper_is_type: false,
                macro_bang: false,
                key_value: false,
                flags: true,
                dashed_idents: false,
                numbers: true,
                calls: false,
            },
            Lang::Toml => Spec {
                line_comment: "#",
                block_comment: None,
                double_quote: true,
                single_quote: true,
                backtick: false,
                keywords: &[],
                constants: &["true", "false"],
                lifetimes: false,
                attributes: false,
                shell_vars: false,
                headers: true,
                upper_is_type: false,
                macro_bang: false,
                key_value: true,
                flags: false,
                dashed_idents: false,
                numbers: true,
                calls: false,
            },
            Lang::Json => Spec {
                line_comment: "//",
                block_comment: Some(("/*", "*/")),
                double_quote: true,
                single_quote: false,
                backtick: false,
                keywords: &[],
                constants: &["true", "false", "null"],
                lifetimes: false,
                attributes: false,
                shell_vars: false,
                headers: false,
                upper_is_type: false,
                macro_bang: false,
                key_value: true,
                flags: false,
                dashed_idents: false,
                numbers: true,
                calls: false,
            },
            Lang::TypeScript => Spec {
                line_comment: "//",
                block_comment: Some(("/*", "*/")),
                double_quote: true,
                single_quote: true,
                backtick: true,
                keywords: TS_KEYWORDS,
                constants: TS_CONSTANTS,
                lifetimes: false,
                attributes: false,
                shell_vars: false,
                headers: false,
                upper_is_type: true,
                macro_bang: false,
                key_value: false,
                flags: false,
                dashed_idents: false,
                numbers: true,
                calls: true,
            },
            Lang::Css => Spec {
                line_comment: "\0",
                block_comment: Some(("/*", "*/")),
                double_quote: true,
                single_quote: true,
                backtick: false,
                keywords: CSS_KEYWORDS,
                constants: CSS_CONSTANTS,
                lifetimes: false,
                attributes: false,
                shell_vars: false,
                headers: false,
                upper_is_type: false,
                macro_bang: false,
                key_value: true,
                flags: false,
                dashed_idents: true,
                numbers: true,
                calls: true,
            },
            Lang::Plain => Spec {
                line_comment: "\0",
                block_comment: None,
                double_quote: false,
                single_quote: false,
                backtick: false,
                keywords: &[],
                constants: &[],
                lifetimes: false,
                attributes: false,
                shell_vars: false,
                headers: false,
                upper_is_type: false,
                macro_bang: false,
                key_value: false,
                flags: false,
                dashed_idents: false,
                numbers: false,
                calls: false,
            },
        }
    }
}

/// Token classes, each mapping to one `github-dark` colour.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Plain,
    Comment,
    Keyword,
    Type,
    Func,
    Str,
    Number,
    Constant,
    Attribute,
    Property,
    Punct,
}

impl Kind {
    /// Hex colour, straight from `github-dark`.
    pub const fn color(self) -> &'static str {
        match self {
            Kind::Plain => "#e6edf3",
            Kind::Comment => "#8b949e",
            Kind::Keyword => "#ff7b72",
            Kind::Type => "#ffa657",
            Kind::Func => "#d2a8ff",
            Kind::Str => "#a5d6ff",
            Kind::Number | Kind::Constant | Kind::Property => "#79c0ff",
            Kind::Attribute => "#7ee787",
            Kind::Punct => "#c9d1d9",
        }
    }

    /// Comments are the only class the theme italicises.
    pub const fn italic(self) -> bool {
        matches!(self, Kind::Comment)
    }
}

/// One highlighted run of text. Never contains a newline — [`highlight`]
/// splits runs at line boundaries so callers can render line-by-line.
#[derive(Clone, Debug)]
pub struct Token {
    pub text: String,
    pub kind: Kind,
}

struct Spec {
    line_comment: &'static str,
    block_comment: Option<(&'static str, &'static str)>,
    double_quote: bool,
    single_quote: bool,
    backtick: bool,
    keywords: &'static [&'static str],
    constants: &'static [&'static str],
    lifetimes: bool,
    attributes: bool,
    shell_vars: bool,
    headers: bool,
    upper_is_type: bool,
    macro_bang: bool,
    key_value: bool,
    flags: bool,
    /// Whether `ident(` should colour as a function call.
    calls: bool,
    /// Whether `-` may appear inside an identifier. CSS needs it for
    /// `--custom-props` and `kebab-case` property names.
    dashed_idents: bool,
    /// Whether numeric literals get their own colour. Off for plain text, so
    /// `text` blocks render with no colour at all.
    numbers: bool,
}

/// Lex `src` and return one `Vec<Token>` per line.
///
/// Trailing newlines are trimmed so a block never renders a blank final row.
pub fn highlight(src: &str, lang: Lang) -> Vec<Vec<Token>> {
    let flat = lex(src.trim_end_matches('\n'), &lang.spec());
    split_lines(flat)
}

fn split_lines(flat: Vec<Token>) -> Vec<Vec<Token>> {
    let mut lines: Vec<Vec<Token>> = vec![Vec::new()];
    for tok in flat {
        let mut parts = tok.text.split('\n');
        // `split` always yields at least one item.
        let first = parts.next().unwrap_or_default();
        if !first.is_empty() {
            lines.last_mut().expect("seeded above").push(Token {
                text: first.to_string(),
                kind: tok.kind,
            });
        }
        for part in parts {
            lines.push(Vec::new());
            if !part.is_empty() {
                lines.last_mut().expect("just pushed").push(Token {
                    text: part.to_string(),
                    kind: tok.kind,
                });
            }
        }
    }
    lines
}

fn lex(src: &str, spec: &Spec) -> Vec<Token> {
    let chars: Vec<char> = src.chars().collect();
    let mut out: Vec<Token> = Vec::new();
    let mut i = 0usize;
    // Column 0 tracking, so TOML `[table]` headers are only recognised at the
    // start of a line and never confused with an array literal.
    let mut at_line_start = true;

    while i < chars.len() {
        let c = chars[i];

        if c == '\n' {
            push(&mut out, "\n", Kind::Plain);
            i += 1;
            at_line_start = true;
            continue;
        }

        if c.is_whitespace() {
            let start = i;
            while i < chars.len() && chars[i] != '\n' && chars[i].is_whitespace() {
                i += 1;
            }
            push(&mut out, &collect(&chars, start, i), Kind::Plain);
            continue;
        }

        // ---- comments ---------------------------------------------------
        if let Some((open, close)) = spec.block_comment {
            if starts_with(&chars, i, open) {
                let start = i;
                i += open.chars().count();
                while i < chars.len() && !starts_with(&chars, i, close) {
                    i += 1;
                }
                i = (i + close.chars().count()).min(chars.len());
                push(&mut out, &collect(&chars, start, i), Kind::Comment);
                at_line_start = false;
                continue;
            }
        }
        if spec.line_comment != "\0" && starts_with(&chars, i, spec.line_comment) {
            let start = i;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            push(&mut out, &collect(&chars, start, i), Kind::Comment);
            continue;
        }

        // ---- rust attributes: #[derive(Clone)] ---------------------------
        if spec.attributes && c == '#' && chars.get(i + 1).is_some_and(|n| *n == '[' || *n == '!') {
            let start = i;
            let mut depth = 0i32;
            while i < chars.len() {
                match chars[i] {
                    '[' => depth += 1,
                    ']' => {
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    '\n' if depth == 0 => break,
                    _ => {}
                }
                i += 1;
            }
            push(&mut out, &collect(&chars, start, i), Kind::Attribute);
            at_line_start = false;
            continue;
        }

        // ---- toml table headers ------------------------------------------
        if spec.headers && at_line_start && c == '[' {
            let start = i;
            while i < chars.len() && chars[i] != ']' && chars[i] != '\n' {
                i += 1;
            }
            if i < chars.len() && chars[i] == ']' {
                i += 1;
            }
            push(&mut out, &collect(&chars, start, i), Kind::Attribute);
            at_line_start = false;
            continue;
        }

        // ---- shell variables: $FOO, ${FOO} -------------------------------
        if spec.shell_vars && c == '$' {
            let start = i;
            i += 1;
            if chars.get(i) == Some(&'{') {
                while i < chars.len() && chars[i] != '}' {
                    i += 1;
                }
                i = (i + 1).min(chars.len());
            } else {
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
            }
            push(&mut out, &collect(&chars, start, i), Kind::Constant);
            at_line_start = false;
            continue;
        }

        // ---- shell flags: --release, -p ------------------------------------
        if spec.flags && c == '-' && !out_ends_with_word(&out) {
            let start = i;
            while i < chars.len()
                && (chars[i] == '-' || chars[i].is_alphanumeric() || chars[i] == '_')
            {
                i += 1;
            }
            if i > start + 1 {
                push(&mut out, &collect(&chars, start, i), Kind::Constant);
                at_line_start = false;
                continue;
            }
            i = start; // a lone `-`; fall through to punctuation
        }

        // ---- strings ------------------------------------------------------
        let quoted = (c == '"' && spec.double_quote)
            || (c == '`' && spec.backtick)
            || (c == '\'' && spec.single_quote && !(spec.lifetimes && is_lifetime(&chars, i)));
        if quoted {
            let start = i;
            // Rust raw strings: r"…", r#"…"#
            let raw_hashes = raw_string_hashes(&chars, i, spec);
            i += 1;
            if let Some(hashes) = raw_hashes {
                let closer: String = std::iter::once('"')
                    .chain(std::iter::repeat('#').take(hashes))
                    .collect();
                while i < chars.len() && !starts_with(&chars, i, &closer) {
                    i += 1;
                }
                i = (i + closer.chars().count()).min(chars.len());
            } else {
                while i < chars.len() {
                    if chars[i] == '\\' {
                        i += 2;
                        continue;
                    }
                    if chars[i] == c {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
            }
            let text = collect(&chars, start, i);
            let kind = if spec.key_value && next_significant(&chars, i) == Some(':') {
                Kind::Property
            } else {
                Kind::Str
            };
            push(&mut out, &text, kind);
            at_line_start = false;
            continue;
        }

        // ---- lifetimes: 'a --------------------------------------------------
        if spec.lifetimes && c == '\'' && is_lifetime(&chars, i) {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            push(&mut out, &collect(&chars, start, i), Kind::Constant);
            at_line_start = false;
            continue;
        }

        // ---- numbers ---------------------------------------------------------
        if c.is_ascii_digit() && spec.numbers {
            let start = i;
            while i < chars.len()
                && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '.')
            {
                // Stop before `..` so Rust ranges do not swallow the operator.
                if chars[i] == '.' && chars.get(i + 1) == Some(&'.') {
                    break;
                }
                i += 1;
            }
            push(&mut out, &collect(&chars, start, i), Kind::Number);
            at_line_start = false;
            continue;
        }

        // ---- identifiers --------------------------------------------------------
        // A leading `-` only opens an identifier in a dashed-ident language and
        // only when a letter or another dash follows, so CSS `--ac-500` lexes as
        // one token while `10px -2px` keeps the minus as punctuation.
        let ident_start = c.is_alphabetic()
            || c == '_'
            || (spec.dashed_idents
                && c == '-'
                && chars
                    .get(i + 1)
                    .is_some_and(|n| n.is_alphabetic() || *n == '-' || *n == '_'));
        if ident_start {
            let start = i;
            i += 1;
            while i < chars.len()
                && (chars[i].is_alphanumeric()
                    || chars[i] == '_'
                    || (spec.dashed_idents && chars[i] == '-'))
            {
                i += 1;
            }
            let word = collect(&chars, start, i);
            let next = next_significant(&chars, i);
            let immediate = chars.get(i).copied();

            let kind = if spec.macro_bang && immediate == Some('!') {
                // `println!` — consume the bang so it colours with the name.
                i += 1;
                Kind::Attribute
            } else if spec.keywords.contains(&word.as_str()) {
                Kind::Keyword
            } else if spec.constants.contains(&word.as_str()) {
                Kind::Constant
            } else if spec.calls && immediate == Some('(') {
                Kind::Func
            } else if spec.key_value && matches!(next, Some('=') | Some(':')) {
                Kind::Property
            } else if spec.upper_is_type && starts_upper(&word) {
                Kind::Type
            } else {
                Kind::Plain
            };
            let text = if kind == Kind::Attribute && spec.macro_bang {
                format!("{word}!")
            } else {
                word
            };
            push(&mut out, &text, kind);
            at_line_start = false;
            continue;
        }

        // ---- punctuation -----------------------------------------------------
        push(&mut out, &c.to_string(), Kind::Punct);
        i += 1;
        at_line_start = false;
    }

    out
}

fn push(out: &mut Vec<Token>, text: &str, kind: Kind) {
    if text.is_empty() {
        return;
    }
    // Coalesce adjacent same-kind runs so the DOM gets fewer spans.
    if let Some(last) = out.last_mut() {
        if last.kind == kind && !text.contains('\n') && !last.text.contains('\n') {
            last.text.push_str(text);
            return;
        }
    }
    out.push(Token {
        text: text.to_string(),
        kind,
    });
}

fn collect(chars: &[char], start: usize, end: usize) -> String {
    chars[start..end.min(chars.len())].iter().collect()
}

fn starts_with(chars: &[char], at: usize, pat: &str) -> bool {
    pat.chars()
        .enumerate()
        .all(|(off, pc)| chars.get(at + off) == Some(&pc))
}

fn starts_upper(word: &str) -> bool {
    word.chars().next().is_some_and(|c| c.is_uppercase())
}

/// The next non-space, non-newline character at or after `from`.
fn next_significant(chars: &[char], from: usize) -> Option<char> {
    chars[from.min(chars.len())..]
        .iter()
        .find(|c| !c.is_whitespace())
        .copied()
}

/// True when the `'` at `at` opens a lifetime rather than a char literal.
/// A char literal is always `'x'` or `'\n'` — i.e. a closing quote two or
/// three positions along.
fn is_lifetime(chars: &[char], at: usize) -> bool {
    if chars.get(at + 1).is_some_and(|c| *c == '\\') {
        return false;
    }
    chars.get(at + 2) != Some(&'\'')
}

/// Number of `#` in a Rust raw-string opener starting at `at`, if any.
fn raw_string_hashes(chars: &[char], at: usize, spec: &Spec) -> Option<usize> {
    if !spec.lifetimes || chars.get(at) != Some(&'"') {
        return None;
    }
    // Walk backwards over `#`s to the `r`.
    let mut back = at;
    let mut hashes = 0usize;
    while back > 0 && chars[back - 1] == '#' {
        back -= 1;
        hashes += 1;
    }
    if back > 0 && chars[back - 1] == 'r' {
        Some(hashes)
    } else {
        None
    }
}

/// Whether the previous emitted token ended in a word character — used to tell
/// a shell flag (`-p`) from a subtraction (`x -1`).
fn out_ends_with_word(out: &[Token]) -> bool {
    out.last()
        .and_then(|t| t.text.chars().last())
        .is_some_and(|c| c.is_alphanumeric() || c == '_')
}

const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "static", "struct", "super", "trait", "type", "union", "unsafe", "use",
    "where", "while", "Self",
];

const RUST_CONSTANTS: &[&str] = &["true", "false", "None", "Some", "Ok", "Err"];

const BASH_KEYWORDS: &[&str] = &[
    "case", "do", "done", "elif", "else", "esac", "export", "fi", "for", "function", "if", "in",
    "local", "return", "set", "then", "unset", "while",
];

const TS_KEYWORDS: &[&str] = &[
    "as",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "declare",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "finally",
    "for",
    "from",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "keyof",
    "let",
    "new",
    "of",
    "readonly",
    "return",
    "satisfies",
    "static",
    "switch",
    "this",
    "throw",
    "try",
    "type",
    "typeof",
    "var",
    "void",
    "while",
    "yield",
];

const TS_CONSTANTS: &[&str] = &["true", "false", "null", "undefined"];

const CSS_KEYWORDS: &[&str] = &["from", "important", "to"];

const CSS_CONSTANTS: &[&str] = &[
    "auto",
    "currentColor",
    "inherit",
    "initial",
    "none",
    "revert",
    "transparent",
    "unset",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds_of(src: &str, lang: Lang, needle: &str) -> Vec<Kind> {
        highlight(src, lang)
            .into_iter()
            .flatten()
            .filter(|t| t.text == needle)
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn splits_into_lines_without_trailing_blank() {
        let lines = highlight("let a = 1;\nlet b = 2;\n", Lang::Rust);
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn rust_keywords_and_types() {
        assert_eq!(
            kinds_of("let x: Duration;", Lang::Rust, "let"),
            [Kind::Keyword]
        );
        assert_eq!(
            kinds_of("let x: Duration;", Lang::Rust, "Duration"),
            [Kind::Type]
        );
    }

    #[test]
    fn lifetimes_are_not_char_literals() {
        let toks: Vec<_> = highlight("struct Ref<'a>(&'a str);", Lang::Rust)
            .into_iter()
            .flatten()
            .collect();
        assert!(
            toks.iter()
                .any(|t| t.text == "'a" && t.kind == Kind::Constant)
        );
        // A real char literal still lexes as a string.
        assert_eq!(kinds_of("let c = 'x';", Lang::Rust, "'x'"), [Kind::Str]);
    }

    #[test]
    fn raw_strings_survive_inner_quotes() {
        let toks: Vec<_> = highlight(r##"let s = r#"a "b" c"#;"##, Lang::Rust)
            .into_iter()
            .flatten()
            .collect();
        assert!(
            toks.iter()
                .any(|t| t.kind == Kind::Str && t.text.contains(r#""b""#))
        );
    }

    #[test]
    fn json_keys_are_properties() {
        let toks: Vec<_> = highlight(r#"{"name": "infinity"}"#, Lang::Json)
            .into_iter()
            .flatten()
            .collect();
        assert!(
            toks.iter()
                .any(|t| t.text == r#""name""# && t.kind == Kind::Property)
        );
        assert!(
            toks.iter()
                .any(|t| t.text == r#""infinity""# && t.kind == Kind::Str)
        );
    }

    #[test]
    fn toml_headers_and_keys() {
        let toks: Vec<_> = highlight("[package]\nname = \"x\"", Lang::Toml)
            .into_iter()
            .flatten()
            .collect();
        assert!(
            toks.iter()
                .any(|t| t.text == "[package]" && t.kind == Kind::Attribute)
        );
        assert!(
            toks.iter()
                .any(|t| t.text == "name" && t.kind == Kind::Property)
        );
    }

    #[test]
    fn bash_flags_not_minus_operator() {
        assert_eq!(
            kinds_of("cargo build --release", Lang::Bash, "--release"),
            [Kind::Constant]
        );
    }

    #[test]
    fn rust_ranges_do_not_swallow_dots() {
        let toks: Vec<_> = highlight("for i in 0..10 {}", Lang::Rust)
            .into_iter()
            .flatten()
            .collect();
        assert!(toks.iter().any(|t| t.text == "0" && t.kind == Kind::Number));
        assert!(
            toks.iter()
                .any(|t| t.text == "10" && t.kind == Kind::Number)
        );
    }

    #[test]
    fn macros_keep_their_bang() {
        assert_eq!(
            kinds_of("println!(\"hi\");", Lang::Rust, "println!"),
            [Kind::Attribute]
        );
    }

    #[test]
    fn plain_lang_emits_no_colour() {
        // Digits and quotes included: `text` blocks must stay entirely uncoloured.
        let toks: Vec<_> = highlight("fn main() { 42 \"str\" }", Lang::Plain)
            .into_iter()
            .flatten()
            .collect();
        assert!(
            toks.iter()
                .all(|t| matches!(t.kind, Kind::Plain | Kind::Punct))
        );
    }

    #[test]
    fn css_custom_properties_are_one_property_token() {
        let toks: Vec<_> = highlight(":root { --ac-500: 16 185 129; }", Lang::Css)
            .into_iter()
            .flatten()
            .collect();
        assert!(
            toks.iter()
                .any(|t| t.text == "--ac-500" && t.kind == Kind::Property)
        );
    }

    #[test]
    fn css_kebab_properties_and_functions() {
        assert_eq!(
            kinds_of(
                "background-color: rgb(0 0 0);",
                Lang::Css,
                "background-color"
            ),
            [Kind::Property]
        );
        assert_eq!(
            kinds_of("a { color: rgb(1); }", Lang::Css, "rgb"),
            [Kind::Func]
        );
    }

    #[test]
    fn css_minus_between_numbers_stays_punctuation() {
        let toks: Vec<_> = highlight("width: calc(100vw - 2rem);", Lang::Css)
            .into_iter()
            .flatten()
            .collect();
        assert!(toks.iter().any(|t| t.text == "-" && t.kind == Kind::Punct));
    }

    #[test]
    fn unknown_tags_fall_back_to_plain() {
        assert_eq!(Lang::from_tag("brainfuck"), Lang::Plain);
        assert_eq!(Lang::from_tag("SCSS"), Lang::Css);
    }
}
