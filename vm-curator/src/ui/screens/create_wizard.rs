//! VM Creation Wizard screens
//!
//! A 5-step wizard for creating new VMs with OS-specific QEMU defaults.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::{App, WizardStep, WizardField, WizardQemuConfig};
use crate::metadata::QemuProfileStore;

/// Render the create wizard based on current step
pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // Wizard dialog size
    let dialog_width = 80.min(area.width.saturating_sub(4));
    let dialog_height = 40.min(area.height.saturating_sub(4));

    let dialog_area = centered_rect(dialog_width, dialog_height, area);
    frame.render_widget(Clear, dialog_area);

    let Some(ref state) = app.wizard_state else {
        return;
    };

    // Render the appropriate step
    match state.step {
        WizardStep::SelectOs => render_step_select_os(app, frame, dialog_area),
        WizardStep::SelectIso => render_step_select_iso(app, frame, dialog_area),
        WizardStep::ConfigureDisk => render_step_configure_disk(app, frame, dialog_area),
        WizardStep::ConfigureQemu => render_step_configure_qemu(app, frame, dialog_area),
        WizardStep::Confirm => render_step_confirm(app, frame, dialog_area),
    }
}

/// Render custom OS entry form
pub fn render_custom_os(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let dialog_width = 70.min(area.width.saturating_sub(4));
    let dialog_height = 30.min(area.height.saturating_sub(4));

    let dialog_area = centered_rect(dialog_width, dialog_height, area);
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Custom OS Entry ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let text = Paragraph::new("Custom OS entry form - Coming soon\n\n[Esc] Cancel")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

/// Render ISO download progress
pub fn render_download(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let dialog_width = 60.min(area.width.saturating_sub(4));
    let dialog_height = 10.min(area.height.saturating_sub(4));

    let dialog_area = centered_rect(dialog_width, dialog_height, area);
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Downloading ISO ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let progress = app.wizard_state.as_ref()
        .map(|s| s.iso_download_progress)
        .unwrap_or(0.0);

    let text = Paragraph::new(format!("Downloading... {:.0}%\n\n[Esc] Cancel", progress * 100.0))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

/// Handle key input for wizard
pub fn handle_key(app: &mut App, key: KeyEvent) -> Result<()> {
    let Some(ref state) = app.wizard_state else {
        return Ok(());
    };

    // Handle step-specific keys
    match state.step {
        WizardStep::SelectOs => handle_step_select_os(app, key),
        WizardStep::SelectIso => handle_step_select_iso(app, key),
        WizardStep::ConfigureDisk => handle_step_configure_disk(app, key),
        WizardStep::ConfigureQemu => handle_step_configure_qemu(app, key),
        WizardStep::Confirm => handle_step_confirm(app, key),
    }
}

/// Handle key input for custom OS form
pub fn handle_custom_os_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.pop_screen();
        }
        _ => {}
    }
    Ok(())
}

/// Handle key input for download screen
pub fn handle_download_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            // Cancel download
            if let Some(ref mut state) = app.wizard_state {
                state.iso_downloading = false;
                state.iso_download_progress = 0.0;
            }
            app.pop_screen();
        }
        _ => {}
    }
    Ok(())
}

// =============================================================================
// Step 1: Select OS
// =============================================================================

fn render_step_select_os(app: &App, frame: &mut Frame, area: Rect) {
    let state = app.wizard_state.as_ref().unwrap();

    let block = Block::default()
        .title(format!(" Create New VM ({}/5) - {} ", state.step.number(), state.step.title()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout: VM name field, OS list, help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),   // VM Name field
            Constraint::Length(1),   // Spacer
            Constraint::Length(1),   // OS list header
            Constraint::Min(10),     // OS list
            Constraint::Length(1),   // Error message
            Constraint::Length(2),   // Help text
        ])
        .split(inner);

    // VM Name input
    let name_editing = matches!(state.editing_field, Some(WizardField::VmName));
    let name_style = if name_editing {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    let name_border = if name_editing {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let name_block = Block::default()
        .title(" VM Name ")
        .borders(Borders::ALL)
        .border_style(name_border);

    let name_text = if state.vm_name.is_empty() {
        Paragraph::new("Enter a name for your VM...")
            .style(Style::default().fg(Color::DarkGray))
            .block(name_block)
    } else {
        Paragraph::new(state.vm_name.as_str())
            .style(name_style)
            .block(name_block)
    };
    frame.render_widget(name_text, chunks[0]);

    // Set cursor position when editing
    if name_editing {
        let cursor_x = chunks[0].x + 1 + state.vm_name.len() as u16;
        let cursor_y = chunks[0].y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }

    // OS list header
    let header = Paragraph::new("Select Operating System:")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    frame.render_widget(header, chunks[2]);

    // OS list (grouped by category)
    render_os_list(app, frame, chunks[3]);

    // Error message
    if let Some(ref error) = state.error_message {
        let error_text = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red));
        frame.render_widget(error_text, chunks[4]);
    }

    // Help text
    let help_text = if name_editing {
        "[Enter] Done editing  [Esc] Cancel"
    } else {
        "[Tab] Edit name  [j/k] Select OS  [Enter] Next  [Esc] Cancel"
    };
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[5]);
}

fn render_os_list(app: &App, frame: &mut Frame, area: Rect) {
    let state = app.wizard_state.as_ref().unwrap();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Build the list of items (categories and OSes)
    let mut lines: Vec<Line> = Vec::new();
    let mut item_index = 0;

    // Get categories in display order
    let category_order = ["windows", "linux", "bsd", "unix", "alternative", "retro", "classic-mac", "macos"];

    for category in &category_order {
        let profiles = app.qemu_profiles.list_by_category(category);
        if profiles.is_empty() {
            continue;
        }

        let is_expanded = state.is_category_expanded(category);
        let is_selected = item_index == state.os_list_selected;

        // Category header
        let expand_icon = if is_expanded { "v" } else { ">" };
        let category_name = QemuProfileStore::category_display_name(category);
        let category_style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        };

        let prefix = if is_selected { "> " } else { "  " };
        lines.push(Line::from(vec![
            Span::styled(prefix, category_style),
            Span::styled(expand_icon, category_style),
            Span::styled(format!(" {}", category_name), category_style),
        ]));

        item_index += 1;

        // OS items (if expanded)
        if is_expanded {
            for (os_id, profile) in &profiles {
                // Filter by search query
                if !state.os_filter.is_empty() {
                    let filter_lower = state.os_filter.to_lowercase();
                    if !profile.display_name.to_lowercase().contains(&filter_lower)
                        && !os_id.to_lowercase().contains(&filter_lower)
                    {
                        continue;
                    }
                }

                let is_os_selected = item_index == state.os_list_selected;
                let is_chosen = state.selected_os.as_ref() == Some(*os_id);

                let os_style = if is_os_selected {
                    Style::default().fg(Color::Yellow)
                } else if is_chosen {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if is_os_selected { "> " } else { "  " };
                let chosen_marker = if is_chosen { "*" } else { " " };
                let summary = profile.summary();

                lines.push(Line::from(vec![
                    Span::styled(prefix, os_style),
                    Span::styled(format!("   {}", chosen_marker), os_style),
                    Span::styled(format!("{}", profile.display_name), os_style),
                    Span::styled(format!("  ({})", summary), Style::default().fg(Color::DarkGray)),
                ]));

                item_index += 1;
            }
        }
    }

    // Add "Custom OS" option at the end
    let is_custom_selected = item_index == state.os_list_selected;
    let custom_style = if is_custom_selected {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Magenta)
    };
    let prefix = if is_custom_selected { "> " } else { "  " };
    lines.push(Line::from(vec![
        Span::styled(prefix, custom_style),
        Span::styled("   Custom OS...", custom_style),
        Span::styled("  (Define your own)", Style::default().fg(Color::DarkGray)),
    ]));

    // Calculate scroll offset
    let visible_height = inner.height as usize;
    let scroll_offset = if state.os_list_selected >= visible_height {
        state.os_list_selected - visible_height + 1
    } else {
        0
    };

    // Render visible portion
    let visible_lines: Vec<Line> = lines
        .into_iter()
        .skip(scroll_offset)
        .take(visible_height)
        .collect();

    let list = Paragraph::new(visible_lines);
    frame.render_widget(list, inner);
}

fn handle_step_select_os(app: &mut App, key: KeyEvent) -> Result<()> {
    let editing_name = app.wizard_state.as_ref()
        .map(|s| matches!(s.editing_field, Some(WizardField::VmName)))
        .unwrap_or(false);

    if editing_name {
        // Text input mode for VM name
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Tab => {
                if let Some(ref mut state) = app.wizard_state {
                    state.editing_field = None;
                    state.update_folder_name();
                }
            }
            KeyCode::Char(c) => {
                if let Some(ref mut state) = app.wizard_state {
                    state.vm_name.push(c);
                }
            }
            KeyCode::Backspace => {
                if let Some(ref mut state) = app.wizard_state {
                    state.vm_name.pop();
                }
            }
            _ => {}
        }
    } else {
        // Normal navigation mode
        match key.code {
            KeyCode::Esc => {
                app.cancel_wizard();
            }
            KeyCode::Tab => {
                // Toggle to name editing
                if let Some(ref mut state) = app.wizard_state {
                    state.editing_field = Some(WizardField::VmName);
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                // Count total items first (immutable borrow)
                let total = count_os_list_items(app);
                // Then mutate
                if let Some(ref mut state) = app.wizard_state {
                    if state.os_list_selected < total.saturating_sub(1) {
                        state.os_list_selected += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(ref mut state) = app.wizard_state {
                    if state.os_list_selected > 0 {
                        state.os_list_selected -= 1;
                    }
                }
            }
            KeyCode::Char(' ') => {
                // Toggle category expansion or select OS
                handle_os_list_action(app, false);
            }
            KeyCode::Enter => {
                // Select OS or expand category, then proceed if valid
                handle_os_list_action(app, true);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Count total items in the OS list (categories + visible OSes + custom)
fn count_os_list_items(app: &App) -> usize {
    let state = app.wizard_state.as_ref().unwrap();
    let category_order = ["windows", "linux", "bsd", "unix", "alternative", "retro", "classic-mac", "macos"];

    let mut count = 0;
    for category in &category_order {
        let profiles = app.qemu_profiles.list_by_category(category);
        if profiles.is_empty() {
            continue;
        }
        count += 1; // Category header
        if state.is_category_expanded(category) {
            // Count visible profiles (with filter)
            for (os_id, profile) in &profiles {
                if !state.os_filter.is_empty() {
                    let filter_lower = state.os_filter.to_lowercase();
                    if !profile.display_name.to_lowercase().contains(&filter_lower)
                        && !os_id.to_lowercase().contains(&filter_lower)
                    {
                        continue;
                    }
                }
                count += 1;
            }
        }
    }
    count += 1; // Custom OS option
    count
}

/// Handle action on OS list item (space to toggle, enter to select and proceed)
fn handle_os_list_action(app: &mut App, proceed: bool) {
    // First, collect all the information we need without holding borrows
    let Some(ref state) = app.wizard_state else {
        return;
    };
    let selected = state.os_list_selected;
    let os_filter = state.os_filter.clone();
    let expanded_categories: Vec<String> = state.expanded_categories.clone();

    let category_order = ["windows", "linux", "bsd", "unix", "alternative", "retro", "classic-mac", "macos"];

    let mut item_index = 0;
    let mut action: Option<OsListAction> = None;

    for category in &category_order {
        let profiles = app.qemu_profiles.list_by_category(category);
        if profiles.is_empty() {
            continue;
        }

        // Category header
        if item_index == selected {
            action = Some(OsListAction::ToggleCategory(category.to_string()));
            break;
        }
        item_index += 1;

        // OS items (if expanded)
        let is_expanded = expanded_categories.iter().any(|c| c == *category);
        if is_expanded {
            for (os_id, profile) in &profiles {
                if !os_filter.is_empty() {
                    let filter_lower = os_filter.to_lowercase();
                    if !profile.display_name.to_lowercase().contains(&filter_lower)
                        && !os_id.to_lowercase().contains(&filter_lower)
                    {
                        continue;
                    }
                }

                if item_index == selected {
                    action = Some(OsListAction::SelectOs(os_id.to_string()));
                    break;
                }
                item_index += 1;
            }
        }

        if action.is_some() {
            break;
        }
    }

    // Check if custom OS was selected (at the end)
    if action.is_none() && item_index == selected {
        action = Some(OsListAction::CustomOs);
    }

    // Now execute the action
    match action {
        Some(OsListAction::ToggleCategory(cat)) => {
            if let Some(ref mut state) = app.wizard_state {
                state.toggle_category(&cat);
            }
        }
        Some(OsListAction::SelectOs(os_id)) => {
            app.wizard_select_os(&os_id);
            if proceed {
                if let Err(e) = app.wizard_next_step() {
                    if let Some(ref mut state) = app.wizard_state {
                        state.error_message = Some(e);
                    }
                }
            }
        }
        Some(OsListAction::CustomOs) => {
            app.wizard_use_custom_os();
        }
        None => {}
    }
}

/// Actions that can be taken on the OS list
enum OsListAction {
    ToggleCategory(String),
    SelectOs(String),
    CustomOs,
}

// =============================================================================
// Step 2: Select ISO
// =============================================================================

fn render_step_select_iso(app: &App, frame: &mut Frame, area: Rect) {
    let state = app.wizard_state.as_ref().unwrap();

    let block = Block::default()
        .title(format!(" Create New VM ({}/5) - {} ", state.step.number(), state.step.title()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),   // OS info
            Constraint::Length(1),   // Spacer
            Constraint::Length(1),   // Header
            Constraint::Min(10),     // Options
            Constraint::Length(1),   // Selected path
            Constraint::Length(2),   // Help
        ])
        .split(inner);

    // OS info
    let os_name = state.selected_os.as_ref()
        .and_then(|id| app.qemu_profiles.get(id))
        .map(|p| p.display_name.as_str())
        .unwrap_or("Custom OS");

    let os_info = Paragraph::new(format!("Operating System: {}", os_name))
        .style(Style::default().fg(Color::White));
    frame.render_widget(os_info, chunks[0]);

    // Header
    let header = Paragraph::new("Installation ISO:")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    frame.render_widget(header, chunks[2]);

    // Options
    let mut lines = Vec::new();

    // Check if this OS has a free ISO URL
    let has_download = state.selected_os.as_ref()
        .and_then(|id| app.qemu_profiles.get(id))
        .and_then(|p| p.iso_url.as_ref())
        .is_some();

    let mut option_idx = 0;

    if has_download {
        let is_selected = state.field_focus == option_idx;
        let style = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        let prefix = if is_selected { "> " } else { "  " };
        lines.push(Line::styled(format!("{}( ) Download ISO from official source", prefix), style));
        option_idx += 1;
    }

    let is_browse_selected = state.field_focus == option_idx;
    let browse_style = if is_browse_selected {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    let browse_prefix = if is_browse_selected { "> " } else { "  " };
    lines.push(Line::styled(format!("{}( ) Browse for local ISO file...", browse_prefix), browse_style));
    option_idx += 1;

    let is_none_selected = state.field_focus == option_idx;
    let none_style = if is_none_selected {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    let none_prefix = if is_none_selected { "> " } else { "  " };
    lines.push(Line::styled(format!("{}( ) No ISO (configure later)", none_prefix), none_style));

    let options = Paragraph::new(lines);
    frame.render_widget(options, chunks[3]);

    // Selected path
    if let Some(ref path) = state.iso_path {
        let path_text = Paragraph::new(format!("Selected: {}", path.display()))
            .style(Style::default().fg(Color::Green));
        frame.render_widget(path_text, chunks[4]);
    }

    // Help
    let help = Paragraph::new("[j/k] Select  [Enter] Choose  [Esc] Back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[5]);
}

fn handle_step_select_iso(app: &mut App, key: KeyEvent) -> Result<()> {
    let has_download = app.wizard_state.as_ref()
        .and_then(|s| s.selected_os.as_ref())
        .and_then(|id| app.qemu_profiles.get(id))
        .and_then(|p| p.iso_url.as_ref())
        .is_some();

    let max_options = if has_download { 3 } else { 2 };

    match key.code {
        KeyCode::Esc => {
            app.wizard_prev_step();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if let Some(ref mut state) = app.wizard_state {
                if state.field_focus < max_options - 1 {
                    state.field_focus += 1;
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if let Some(ref mut state) = app.wizard_state {
                if state.field_focus > 0 {
                    state.field_focus -= 1;
                }
            }
        }
        KeyCode::Enter => {
            let focus = app.wizard_state.as_ref().map(|s| s.field_focus).unwrap_or(0);
            let option_offset = if has_download { 0 } else { 1 };

            match focus + option_offset {
                0 => {
                    // Download ISO
                    // For now, just go to next step
                    let _ = app.wizard_next_step();
                }
                1 => {
                    // Browse for ISO - open file browser
                    app.load_file_browser();
                    app.push_screen(crate::app::Screen::FileBrowser);
                }
                2 => {
                    // No ISO
                    if let Some(ref mut state) = app.wizard_state {
                        state.iso_path = None;
                    }
                    let _ = app.wizard_next_step();
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

// =============================================================================
// Step 3: Configure Disk
// =============================================================================

fn render_step_configure_disk(app: &App, frame: &mut Frame, area: Rect) {
    let state = app.wizard_state.as_ref().unwrap();

    let block = Block::default()
        .title(format!(" Create New VM ({}/5) - {} ", state.step.number(), state.step.title()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),   // Header
            Constraint::Length(1),   // Spacer
            Constraint::Length(3),   // Disk size input
            Constraint::Length(1),   // Spacer
            Constraint::Min(6),      // Disk info
            Constraint::Length(2),   // Help
        ])
        .split(inner);

    // Header
    let header = Paragraph::new("Disk Configuration")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    frame.render_widget(header, chunks[0]);

    // Disk size input
    let editing = matches!(state.editing_field, Some(WizardField::DiskSize));
    let size_style = if editing {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    let border_style = if editing {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let recommended = app.wizard_selected_profile()
        .map(|p| p.disk_size_gb)
        .unwrap_or(32);

    let size_block = Block::default()
        .title(format!(" Disk Size (Recommended: {} GB) ", recommended))
        .borders(Borders::ALL)
        .border_style(border_style);

    let size_text = Paragraph::new(format!("{} GB", state.disk_size_gb))
        .style(size_style)
        .block(size_block);
    frame.render_widget(size_text, chunks[2]);

    // Disk info box
    let info_block = Block::default()
        .title(" Disk Info ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));

    let disk_path = app.wizard_vm_path()
        .map(|p| p.join(format!("{}.qcow2", state.folder_name)))
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "~/vm-space/<vm-name>/<vm-name>.qcow2".to_string());

    let info_text = vec![
        Line::from(vec![
            Span::styled("Format: ", Style::default().fg(Color::Yellow)),
            Span::raw("qcow2 (copy-on-write, snapshots supported)"),
        ]),
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Yellow)),
            Span::raw("Expandable (only uses space as needed)"),
        ]),
        Line::from(vec![
            Span::styled("Location: ", Style::default().fg(Color::Yellow)),
            Span::raw(disk_path),
        ]),
    ];

    let info = Paragraph::new(info_text)
        .block(info_block)
        .wrap(Wrap { trim: false });
    frame.render_widget(info, chunks[4]);

    // Help
    let help_text = if editing {
        "[Enter] Done  [Backspace] Delete  [0-9] Enter size"
    } else {
        "[Tab] Edit size  [Left/Right] Adjust  [Enter] Next  [Esc] Back"
    };
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[5]);
}

fn handle_step_configure_disk(app: &mut App, key: KeyEvent) -> Result<()> {
    let editing = app.wizard_state.as_ref()
        .map(|s| matches!(s.editing_field, Some(WizardField::DiskSize)))
        .unwrap_or(false);

    if editing {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Tab => {
                if let Some(ref mut state) = app.wizard_state {
                    state.editing_field = None;
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(ref mut state) = app.wizard_state {
                    let new_size = state.disk_size_gb
                        .saturating_mul(10)
                        .saturating_add((c as u32) - ('0' as u32));
                    if new_size <= 10000 {
                        state.disk_size_gb = new_size;
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(ref mut state) = app.wizard_state {
                    state.disk_size_gb /= 10;
                }
            }
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Esc => {
                app.wizard_prev_step();
            }
            KeyCode::Tab => {
                if let Some(ref mut state) = app.wizard_state {
                    state.editing_field = Some(WizardField::DiskSize);
                }
            }
            KeyCode::Left => {
                if let Some(ref mut state) = app.wizard_state {
                    state.disk_size_gb = state.disk_size_gb.saturating_sub(8).max(1);
                }
            }
            KeyCode::Right => {
                if let Some(ref mut state) = app.wizard_state {
                    state.disk_size_gb = (state.disk_size_gb + 8).min(10000);
                }
            }
            KeyCode::Enter => {
                let _ = app.wizard_next_step();
            }
            _ => {}
        }
    }
    Ok(())
}

// =============================================================================
// Step 4: Configure QEMU
// =============================================================================

fn render_step_configure_qemu(app: &App, frame: &mut Frame, area: Rect) {
    let state = app.wizard_state.as_ref().unwrap();

    let block = Block::default()
        .title(format!(" Create New VM ({}/5) - {} ", state.step.number(), state.step.title()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),   // Header
            Constraint::Length(1),   // Spacer
            Constraint::Min(20),     // Settings
            Constraint::Length(2),   // Help
        ])
        .split(inner);

    // Header
    let header = Paragraph::new("QEMU Configuration")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    frame.render_widget(header, chunks[0]);

    // Settings display
    let config = &state.qemu_config;
    let mut lines = Vec::new();

    lines.push(Line::from(vec![
        Span::styled("Memory: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{} MB", config.memory_mb)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("CPU Cores: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{}", config.cpu_cores)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("CPU Model: ", Style::default().fg(Color::Yellow)),
        Span::raw(config.cpu_model.as_deref().unwrap_or("default")),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Machine: ", Style::default().fg(Color::Yellow)),
        Span::raw(config.machine.as_deref().unwrap_or("default")),
    ]));

    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("VGA: ", Style::default().fg(Color::Yellow)),
        Span::raw(&config.vga),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Display: ", Style::default().fg(Color::Yellow)),
        Span::raw(&config.display),
    ]));

    let audio_str = config.audio.join(", ");
    lines.push(Line::from(vec![
        Span::styled("Audio: ", Style::default().fg(Color::Yellow)),
        Span::raw(if audio_str.is_empty() { "None" } else { &audio_str }),
    ]));

    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("Disk Interface: ", Style::default().fg(Color::Yellow)),
        Span::raw(&config.disk_interface),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Network: ", Style::default().fg(Color::Yellow)),
        Span::raw(&config.network_model),
    ]));

    lines.push(Line::from(""));

    // Toggles
    let features: Vec<&str> = [
        ("KVM", config.enable_kvm),
        ("UEFI", config.uefi),
        ("TPM", config.tpm),
        ("USB Tablet", config.usb_tablet),
        ("RTC Local", config.rtc_localtime),
    ]
    .iter()
    .filter(|(_, enabled)| *enabled)
    .map(|(name, _)| *name)
    .collect();

    lines.push(Line::from(vec![
        Span::styled("Features: ", Style::default().fg(Color::Yellow)),
        Span::raw(features.join(", ")),
    ]));

    let settings = Paragraph::new(lines)
        .wrap(Wrap { trim: false });
    frame.render_widget(settings, chunks[2]);

    // Help
    let help = Paragraph::new("[r] Reset to defaults  [Enter] Next  [Esc] Back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);
}

fn handle_step_configure_qemu(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.wizard_prev_step();
        }
        KeyCode::Enter => {
            let _ = app.wizard_next_step();
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            // Reset to profile defaults
            if let Some(profile) = app.wizard_selected_profile().cloned() {
                if let Some(ref mut state) = app.wizard_state {
                    state.qemu_config = WizardQemuConfig::from_profile(&profile);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

// =============================================================================
// Step 5: Confirm
// =============================================================================

fn render_step_confirm(app: &App, frame: &mut Frame, area: Rect) {
    let state = app.wizard_state.as_ref().unwrap();

    let block = Block::default()
        .title(format!(" Create New VM ({}/5) - {} ", state.step.number(), state.step.title()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),   // Header
            Constraint::Length(1),   // Spacer
            Constraint::Min(15),     // Summary
            Constraint::Length(3),   // Auto-launch toggle
            Constraint::Length(1),   // Error
            Constraint::Length(2),   // Help
        ])
        .split(inner);

    // Header
    let header = Paragraph::new("Summary")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    frame.render_widget(header, chunks[0]);

    // Summary
    let os_name = state.selected_os.as_ref()
        .and_then(|id| app.qemu_profiles.get(id))
        .map(|p| p.display_name.as_str())
        .unwrap_or("Custom OS");

    let vm_path = app.wizard_vm_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let iso_str = state.iso_path.as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "None".to_string());

    let config = &state.qemu_config;

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("VM Name:        ", Style::default().fg(Color::Yellow)),
        Span::raw(&state.vm_name),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Folder:         ", Style::default().fg(Color::Yellow)),
        Span::raw(vm_path),
    ]));
    lines.push(Line::from(vec![
        Span::styled("OS Type:        ", Style::default().fg(Color::Yellow)),
        Span::raw(os_name),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Disk:           ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{} GB qcow2 (expandable)", state.disk_size_gb)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("ISO:            ", Style::default().fg(Color::Yellow)),
        Span::raw(iso_str),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Hardware:       ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{} cores, {} MB RAM", config.cpu_cores, config.memory_mb)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Graphics:       ", Style::default().fg(Color::Yellow)),
        Span::raw(&config.vga),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Audio:          ", Style::default().fg(Color::Yellow)),
        Span::raw(config.audio.first().cloned().unwrap_or_else(|| "None".to_string())),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Network:        ", Style::default().fg(Color::Yellow)),
        Span::raw(&config.network_model),
    ]));

    let accel = if config.enable_kvm { "KVM enabled" } else { "No acceleration" };
    lines.push(Line::from(vec![
        Span::styled("Acceleration:   ", Style::default().fg(Color::Yellow)),
        Span::raw(accel),
    ]));

    let summary = Paragraph::new(lines)
        .wrap(Wrap { trim: false });
    frame.render_widget(summary, chunks[2]);

    // Auto-launch toggle
    let launch_box = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));
    let checkbox = if state.auto_launch { "[x]" } else { "[ ]" };
    let launch_text = Paragraph::new(format!("{} Launch VM in install mode after creation", checkbox))
        .style(Style::default().fg(Color::White))
        .block(launch_box);
    frame.render_widget(launch_text, chunks[3]);

    // Error
    if let Some(ref error) = state.error_message {
        let error_text = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red));
        frame.render_widget(error_text, chunks[4]);
    }

    // Help
    let help = Paragraph::new("[Enter] Create VM  [Space] Toggle launch  [Esc] Back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[5]);
}

fn handle_step_confirm(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.wizard_prev_step();
        }
        KeyCode::Char(' ') => {
            if let Some(ref mut state) = app.wizard_state {
                state.auto_launch = !state.auto_launch;
            }
        }
        KeyCode::Enter => {
            // Create the VM
            // TODO: Implement actual VM creation
            app.set_status("VM creation not yet implemented - coming soon!");
            app.cancel_wizard();
        }
        _ => {}
    }
    Ok(())
}

// =============================================================================
// Utility
// =============================================================================

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}
