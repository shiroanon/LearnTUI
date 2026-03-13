use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
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
    let mut lines: Vec<Line<'static>> = Vec::new();

    let parser = Parser::new(input);
    let mut in_code_block = false;
    let mut current_language: Option<String> = None;
    let mut current_block = String::new();

    let mut current_line_spans = Vec::new();

    // Table state variables
    let mut in_table = false;
    let mut table_alignments = Vec::new();
    let mut table_rows: Vec<Vec<Vec<Span<'static>>>> = Vec::new();
    let mut current_row: Vec<Vec<Span<'static>>> = Vec::new();
    let mut current_cell_spans: Vec<Span<'static>> = Vec::new();

    let base_style = Style::default().fg(Color::Rgb(235, 240, 245));
    let mut style_stack = vec![base_style];
    let mut list_index_stack = Vec::new();

    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    macro_rules! flush_line {
        () => {
            if !current_line_spans.is_empty() {
                lines.push(Line::from(std::mem::take(&mut current_line_spans)));
            }
        };
    }

    macro_rules! push_blank_line {
        () => {
            if lines.last().map_or(false, |l| !l.spans.is_empty()) {
                lines.push(Line::from(vec![]));
            }
        };
    }

    for event in parser {
        let current_style = *style_stack.last().unwrap_or(&base_style);

        match event {
            Event::Start(tag) => {
                let mut new_style = current_style;

                match &tag {
                    Tag::CodeBlock(kind) => {
                        in_code_block = true;
                        current_block.clear();
                        if let CodeBlockKind::Fenced(lang) = kind {
                            let lang_str = lang.to_string();
                            let clean_lang = lang_str
                                .split(',')
                                .next()
                                .unwrap_or(&lang_str)
                                .trim()
                                .to_string();
                            current_language = Some(clean_lang);
                        } else {
                            current_language = None;
                        }
                    }
                    Tag::Table(alignments) => {
                        in_table = true;
                        table_alignments = alignments.clone();
                        table_rows.clear();
                    }
                    Tag::TableHead | Tag::TableRow => {
                        current_row.clear();
                    }
                    Tag::TableCell => {
                        current_cell_spans.clear();
                    }
                    Tag::Heading { level, .. } => {
                        let (heading_color, modifier) = match level {
                            pulldown_cmark::HeadingLevel::H1 => (Color::Rgb(255, 110, 150), Modifier::BOLD | Modifier::UNDERLINED),
                            pulldown_cmark::HeadingLevel::H2 => (Color::Rgb(110, 255, 150), Modifier::BOLD),
                            pulldown_cmark::HeadingLevel::H3 => (Color::Rgb(110, 200, 255), Modifier::BOLD),
                            pulldown_cmark::HeadingLevel::H4 => (Color::Rgb(255, 220, 110), Modifier::BOLD),
                            _ => (Color::Rgb(200, 200, 200), Modifier::BOLD),
                        };
                        new_style = current_style.fg(heading_color).add_modifier(modifier);

                        current_line_spans.push(Span::styled(
                            format!("{} ", "#".repeat(*level as usize)),
                            Style::default().fg(heading_color).add_modifier(Modifier::BOLD),
                        ));
                    }
                    Tag::Strong => new_style = current_style.add_modifier(Modifier::BOLD).fg(Color::Rgb(255, 215, 0)),
                    Tag::Emphasis => new_style = current_style.add_modifier(Modifier::ITALIC).fg(Color::Rgb(200, 180, 255)),
                    Tag::Strikethrough => new_style = current_style.add_modifier(Modifier::CROSSED_OUT).fg(Color::Rgb(150, 150, 150)),
                    Tag::Link { .. } => new_style = current_style.fg(Color::Rgb(80, 180, 255)).add_modifier(Modifier::UNDERLINED),
                    Tag::Image { .. } => {
                        new_style = current_style.fg(Color::Rgb(255, 150, 255)).add_modifier(Modifier::ITALIC);
                        current_line_spans.push(Span::styled("🖼  ", Style::default()));
                    }
                    Tag::List(start_index) => list_index_stack.push(*start_index),
                    Tag::Item => {
                        let indent = "  ".repeat(list_index_stack.len().saturating_sub(1));
                        let prefix = if let Some(Some(idx)) = list_index_stack.last_mut() {
                            let s = format!("{}{} ", indent, idx);
                            *idx += 1;
                            s
                        } else {
                            format!("{}• ", indent)
                        };
                        current_line_spans.push(Span::styled(prefix, current_style.fg(Color::Rgb(255, 150, 100)).add_modifier(Modifier::BOLD)));
                    }
                    Tag::BlockQuote(_) => {
                        new_style = current_style.fg(Color::Rgb(180, 190, 200)).add_modifier(Modifier::ITALIC);
                        current_line_spans.push(Span::styled("▍ ", Style::default().fg(Color::Rgb(100, 250, 150))));
                    }
                    _ => {}
                }
                style_stack.push(new_style);
            }
            Event::End(tag_end) => {
                style_stack.pop();

                match tag_end {
                    TagEnd::CodeBlock => {
                        in_code_block = false;

                        let syntax = current_language
                            .as_deref()
                            .and_then(|lang| ps.find_syntax_by_token(lang))
                            .unwrap_or_else(|| ps.find_syntax_plain_text());

                        let mut h = HighlightLines::new(syntax, theme);

                        let bg_color = Color::Rgb(20, 22, 28);
                        let border_color = Color::Rgb(100, 120, 150);

                        let max_line_width = current_block
                            .lines()
                            .map(|line| line.replace('\t', "    ").chars().count())
                            .max()
                            .unwrap_or(0);

                        let lang_str = current_language.clone().unwrap_or_default();
                        let top_prefix = if lang_str.is_empty() {
                            "╭───".to_string()
                        } else {
                            format!("╭── {} ", lang_str)
                        };

                        let min_content_width = top_prefix.chars().count().saturating_sub(2);
                        
                        // Perfectly scale the box to fit the widest line of code
                        let total_width = std::cmp::max(max_line_width + 2, min_content_width + 2);
                        let full_box_width = total_width + 3;

                        let top_dashes = full_box_width.saturating_sub(top_prefix.chars().count() + 1);
                        let top_border = format!("{}{}╮", top_prefix, "─".repeat(top_dashes));

                        lines.push(
                            Line::from(Span::styled(top_border, Style::default().fg(border_color).bg(bg_color)))
                            .style(Style::default().bg(bg_color)),
                        );

                        for line in LinesWithEndings::from(&current_block) {
                            let mut spans = vec![Span::styled("│ ", Style::default().fg(border_color).bg(bg_color))];

                            let mut line_char_count = 0;
                            let ranges: Vec<(SyntectStyle, &str)> = h.highlight_line(line, &ps).unwrap_or_default();

                            for (style, text) in ranges {
                                let clean_text_raw = text.trim_end_matches(&['\n', '\r'][..]);
                                
                                // Process the text even if empty to ensure the line padding logic still executes!
                                if !clean_text_raw.is_empty() {
                                    let clean_text = clean_text_raw.replace('\t', "    ");
                                    line_char_count += clean_text.chars().count();

                                    let boost = 1.5;
                                    let r = ((style.foreground.r as f32 * boost).min(255.0)) as u8;
                                    let g = ((style.foreground.g as f32 * boost).min(255.0)) as u8;
                                    let b = ((style.foreground.b as f32 * boost).min(255.0)) as u8;
                                    let fg_color = Color::Rgb(r, g, b);

                                    let mut modifier = Modifier::empty();
                                    if style.font_style.contains(syntect::highlighting::FontStyle::BOLD) { modifier |= Modifier::BOLD; }
                                    if style.font_style.contains(syntect::highlighting::FontStyle::ITALIC) { modifier |= Modifier::ITALIC; }

                                    spans.push(Span::styled(clean_text, Style::default().fg(fg_color).bg(bg_color).add_modifier(modifier)));
                                }
                            }

                            // Use NON-BREAKING SPACES (\u{00A0}) so Ratatui's paragraph renderer doesn't aggressively trim our background color!
                            let padding_spaces = total_width.saturating_sub(line_char_count);
                            let right_padding = "\u{00A0}".repeat(padding_spaces);
                            
                            spans.push(Span::styled(right_padding, Style::default().bg(bg_color)));
                            spans.push(Span::styled("│", Style::default().fg(border_color).bg(bg_color)));

                            lines.push(Line::from(spans).style(Style::default().bg(bg_color)));
                        }

                        let bottom_dashes = full_box_width.saturating_sub(2);
                        let bottom_border = format!("╰{}╯", "─".repeat(bottom_dashes));

                        lines.push(
                            Line::from(Span::styled(bottom_border, Style::default().fg(border_color).bg(bg_color)))
                            .style(Style::default().bg(bg_color)),
                        );

                        current_block.clear();
                        current_language = None;
                        push_blank_line!();
                    }
                    TagEnd::Table => {
                        in_table = false;
                        
                        // Calculate maximum widths per column to properly format the table grid
                        let mut col_widths = vec![0; table_alignments.len()];
                        for row in &table_rows {
                            for (i, cell) in row.iter().enumerate() {
                                if i < col_widths.len() {
                                    let w: usize = cell.iter().map(|s| s.content.chars().count()).sum();
                                    col_widths[i] = col_widths[i].max(w);
                                }
                            }
                        }

                        let border_color = Color::Rgb(100, 120, 150);
                        let header_bg = Color::Rgb(35, 40, 50);
                        let cell_bg = Color::Rgb(20, 22, 28);

                        // Helper closure to build top/middle/bottom horizontal dividers
                        let make_sep = |left: &str, mid: &str, right: &str| -> Line<'static> {
                            let mut s = String::from(left);
                            for (i, w) in col_widths.iter().enumerate() {
                                s.push_str(&"─".repeat(*w + 2));
                                if i < col_widths.len() - 1 { s.push_str(mid); }
                            }
                            s.push_str(right);
                            Line::from(Span::styled(s, Style::default().fg(border_color)))
                        };

                        lines.push(make_sep("╭", "┬", "╮"));
                        
                        for (row_idx, row) in table_rows.iter().enumerate() {
                            let mut spans = vec![Span::styled("│", Style::default().fg(border_color))];
                            
                            let is_header = row_idx == 0;
                            let bg = if is_header { header_bg } else { cell_bg };

                            for i in 0..col_widths.len() {
                                let w = col_widths[i];
                                let cell = row.get(i);
                                
                                spans.push(Span::styled("\u{00A0}", Style::default().bg(bg)));
                                
                                let mut cell_content_len = 0;
                                if let Some(cell_spans) = cell {
                                    for span in cell_spans {
                                        cell_content_len += span.content.chars().count();
                                        let mut new_span = span.clone();
                                        new_span.style = new_span.style.bg(bg);
                                        if is_header {
                                            new_span.style = new_span.style.add_modifier(Modifier::BOLD).fg(Color::Rgb(255, 215, 0));
                                        }
                                        spans.push(new_span);
                                    }
                                }
                                
                                let pad_len = w.saturating_sub(cell_content_len);
                                spans.push(Span::styled("\u{00A0}".repeat(pad_len + 1), Style::default().bg(bg)));
                                spans.push(Span::styled("│", Style::default().fg(border_color)));
                            }
                            
                            lines.push(Line::from(spans));

                            if is_header {
                                lines.push(make_sep("├", "┼", "┤"));
                            }
                        }
                        
                        lines.push(make_sep("╰", "┴", "╯"));
                        push_blank_line!();
                    }
                    TagEnd::TableHead | TagEnd::TableRow => {
                        if !current_row.is_empty() { table_rows.push(std::mem::take(&mut current_row)); }
                    }
                    TagEnd::TableCell => {
                        current_row.push(std::mem::take(&mut current_cell_spans));
                    }
                    TagEnd::Heading(_) => {
                        flush_line!();
                        push_blank_line!();
                    }
                    TagEnd::Paragraph => {
                        flush_line!();
                        if list_index_stack.is_empty() { push_blank_line!(); }
                    }
                    TagEnd::Item => flush_line!(),
                    TagEnd::List(_) => {
                        list_index_stack.pop();
                        if list_index_stack.is_empty() { push_blank_line!(); }
                    }
                    TagEnd::BlockQuote(_) => {
                        flush_line!();
                        if list_index_stack.is_empty() { push_blank_line!(); }
                    }
                    _ => {}
                }
            }
            Event::Text(text) => {
                if in_table {
                    current_cell_spans.push(Span::styled(text.into_string(), current_style));
                } else if in_code_block {
                    current_block.push_str(&text);
                } else {
                    current_line_spans.push(Span::styled(text.into_string(), current_style));
                }
            }
            Event::Code(text) => {
                let code_style = Style::default().fg(Color::Rgb(255, 150, 200)).bg(Color::Rgb(45, 40, 50));
                if in_table {
                    current_cell_spans.push(Span::styled(text.into_string(), code_style));
                } else {
                    current_line_spans.push(Span::styled(text.into_string(), code_style));
                }
            }
            Event::SoftBreak => {
                if in_table {
                    current_cell_spans.push(Span::styled(" ", current_style));
                } else if !in_code_block {
                    current_line_spans.push(Span::styled(" ", current_style));
                }
            }
            Event::HardBreak => {
                if in_table {
                    current_cell_spans.push(Span::styled(" ", current_style));
                } else if !in_code_block {
                    flush_line!();
                }
            }
            Event::Rule => {
                flush_line!();
                lines.push(Line::from(vec![Span::styled("─".repeat(40), Style::default().fg(Color::Rgb(100, 100, 100)))]));
                push_blank_line!();
            }
            _ => {}
        }
    }

    flush_line!();
    lines
}