use syntect::{
    easy::HighlightLines,
    highlighting::{Style, ThemeSet},
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

pub fn highlight(code: &str) -> Vec<String> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps.find_syntax_by_extension("py").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    let mut output = vec![];

    for line in LinesWithEndings::from(code) {
        let mut ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
        if let Some((_, s)) = ranges.last_mut() {
            *s = s.strip_suffix("\r\n").or(s.strip_suffix('\n')).unwrap_or(s);
        }

        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        output.push(escaped);
    }
    output
}
