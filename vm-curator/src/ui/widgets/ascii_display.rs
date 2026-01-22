use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::metadata::OsInfo;

/// ASCII art and info display widget
pub struct AsciiInfoWidget<'a> {
    pub ascii_art: &'a str,
    pub os_info: Option<&'a OsInfo>,
    pub vm_name: &'a str,
}

impl<'a> AsciiInfoWidget<'a> {
    pub fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        // Split the area: ASCII art on top, info below
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // ASCII art
                Constraint::Length(3),  // Name and details
                Constraint::Min(3),     // Blurb
            ])
            .split(inner);

        // Render ASCII art
        let ascii = Paragraph::new(self.ascii_art.trim_start_matches('\n'))
            .style(Style::default().fg(Color::Green));
        ascii.render(chunks[0], buf);

        // Render name and details
        if let Some(info) = self.os_info {
            let details = vec![
                Line::from(vec![
                    Span::styled(&info.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled(&info.publisher, Style::default().fg(Color::Gray)),
                    Span::raw(" | "),
                    Span::styled(&info.release_date, Style::default().fg(Color::Gray)),
                    Span::raw(" | "),
                    Span::styled(&info.architecture, Style::default().fg(Color::Gray)),
                ]),
            ];
            let details_para = Paragraph::new(details);
            details_para.render(chunks[1], buf);

            // Render blurb
            if !info.blurb.short.is_empty() {
                let blurb = Paragraph::new(info.blurb.short.as_str())
                    .style(Style::default().fg(Color::White))
                    .wrap(Wrap { trim: true });
                blurb.render(chunks[2], buf);
            }
        } else {
            // Just show the VM name
            let name = Paragraph::new(self.vm_name)
                .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
            name.render(chunks[1], buf);
        }
    }
}

/// Detailed info display (for the info screen)
pub struct DetailedInfoWidget<'a> {
    pub os_info: Option<&'a OsInfo>,
    pub vm_name: &'a str,
}

impl<'a> DetailedInfoWidget<'a> {
    pub fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(format!(" {} - Details ", self.vm_name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        if let Some(info) = self.os_info {
            let mut text = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::Yellow)),
                    Span::raw(&info.name),
                ]),
                Line::from(vec![
                    Span::styled("Publisher: ", Style::default().fg(Color::Yellow)),
                    Span::raw(&info.publisher),
                ]),
                Line::from(vec![
                    Span::styled("Released: ", Style::default().fg(Color::Yellow)),
                    Span::raw(&info.release_date),
                ]),
                Line::from(vec![
                    Span::styled("Architecture: ", Style::default().fg(Color::Yellow)),
                    Span::raw(&info.architecture),
                ]),
                Line::from(""),
            ];

            // Add long description
            if !info.blurb.long.is_empty() {
                text.push(Line::from(Span::styled(
                    "About",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )));
                for line in info.blurb.long.lines() {
                    text.push(Line::from(line.to_string()));
                }
                text.push(Line::from(""));
            }

            // Add fun facts
            if !info.fun_facts.is_empty() {
                text.push(Line::from(Span::styled(
                    "Fun Facts",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )));
                for fact in &info.fun_facts {
                    text.push(Line::from(format!("• {}", fact)));
                }
            }

            let para = Paragraph::new(text)
                .wrap(Wrap { trim: true });
            para.render(inner, buf);
        } else {
            let text = Paragraph::new("No detailed information available for this VM.")
                .style(Style::default().fg(Color::Gray));
            text.render(inner, buf);
        }
    }
}
