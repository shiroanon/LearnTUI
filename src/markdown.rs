use pulldown_cmark::{Event, Parser, Tag, TagEnd, CodeBlockKind};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{Style as SyntectStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

pub fn parse_to_lines(input: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    let parser = Parser::new(input);
    let mut in_code_block = false;
    let mut current_language: Option<String> = None;
    let mut current_block = String::new();
    
    let mut current_line_spans = Vec::new();
    
    let base_style = Style::default().fg(Color::Gray);

    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"]; // Good terminal-like theme

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                current_block.clear();
                if let CodeBlockKind::Fenced(lang) = kind {
                    let lang_str = lang.into_string();
                    let clean_lang = lang_str.split(',').next().unwrap_or(&lang_str).trim().to_string();
                    current_language = Some(clean_lang);
                } else {
                    current_language = None;
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                
                // Highlight the accumulated code block
                let syntax = current_language
                    .as_deref()
                    .and_then(|lang| ps.find_syntax_by_token(lang))
                    .unwrap_or_else(|| ps.find_syntax_plain_text());
                    
                let mut h = HighlightLines::new(syntax, theme);
                
                for line in LinesWithEndings::from(&current_block) {
                    let ranges: Vec<(SyntectStyle, &str)> = h.highlight_line(line, &ps).unwrap_or_default();
                    let spans: Vec<Span> = ranges.into_iter().filter_map(|(style, text)| {
                        let clean_text = text.trim_end_matches(&['\n', '\r'][..]);
                        if clean_text.is_empty() { return None; }
                        
                        let fg_color = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                        let bg_color = Color::Rgb(20, 20, 20);
                        
                        Some(Span::styled(
                            clean_text.to_string(), 
                            Style::default()
                                .fg(fg_color)
                                .bg(bg_color)
                                .add_modifier(
                                    if style.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
                                        Modifier::BOLD
                                    } else if style.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
                                        Modifier::ITALIC
                                    } else {
                                        Modifier::empty()
                                    }
                                )
                        ))
                    }).collect();
                    
                    // Setting bg on the Line itself makes Ratatui fill the whole row width.
                    let code_line = Line::from(spans)
                        .style(Style::default().bg(Color::Rgb(20, 20, 20)));
                    lines.push(code_line);
                }
                
                current_block.clear();
                current_language = None;
                lines.push(Line::from(vec![])); // empty line after block
            }
            Event::Text(text) => {
                if in_code_block {
                    current_block.push_str(&text);
                } else {
                    current_line_spans.push(Span::styled(text.into_string(), base_style));
                }
            }
            Event::Code(text) => {
                // Inline code formatting
                current_line_spans.push(Span::styled(
                    text.into_string(), 
                    Style::default().fg(Color::Yellow).bg(Color::Rgb(40, 40, 40))
                ));
            }
            Event::Start(Tag::Heading { level, .. }) => {
                // Heading formatting
                let heading_color = match level {
                    pulldown_cmark::HeadingLevel::H1 => Color::Magenta,
                    pulldown_cmark::HeadingLevel::H2 => Color::LightMagenta,
                    pulldown_cmark::HeadingLevel::H3 => Color::Cyan,
                    _ => Color::LightCyan,
                };
                current_line_spans.push(Span::styled(
                    format!("{} ", "#".repeat(level as usize)),
                    Style::default().fg(heading_color).add_modifier(Modifier::BOLD)
                ));
            }
            Event::End(TagEnd::Heading(_)) => {
               // Apply style to the accumulated heading text
               for span in &mut current_line_spans {
                   span.style = span.style.add_modifier(Modifier::BOLD).fg(Color::LightMagenta);
               }
               if !in_code_block {
                   lines.push(Line::from(current_line_spans.clone()));
                   current_line_spans.clear();
                   lines.push(Line::from(vec![])); // empty line after heading
               }
            }
            Event::SoftBreak => {
                if !in_code_block {
                    current_line_spans.push(Span::styled(" ", base_style));
                }
            }
            Event::HardBreak => {
                if !in_code_block {
                    lines.push(Line::from(current_line_spans.clone()));
                    current_line_spans.clear();
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if !in_code_block {
                    lines.push(Line::from(current_line_spans.clone()));
                    current_line_spans.clear();
                    lines.push(Line::from(vec![])); // empty line between paragraphs
                }
            }
            _ => {}
        }
    }
    
    if !current_line_spans.is_empty() {
        lines.push(Line::from(current_line_spans));
    }

    lines
}
