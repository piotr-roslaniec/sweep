use super::{RiskLevel, ScanResult};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, stdout};
use std::time::Duration;

/// Guard that ensures terminal is cleaned up on panic or drop
struct TerminalCleanupGuard;

impl TerminalCleanupGuard {
    fn new() -> Self {
        // Set up panic hook to clean up terminal
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            // Clean up terminal before panicking
            let _ = disable_raw_mode();
            let _ = execute!(stdout(), LeaveAlternateScreen);
            // Call the original panic hook
            original_hook(panic_info);
        }));
        TerminalCleanupGuard
    }
}

impl Drop for TerminalCleanupGuard {
    fn drop(&mut self) {
        // Ensure terminal is cleaned up when guard is dropped
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
}

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

#[derive(Debug, Clone)]
pub struct SelectableItem {
    pub scan_result: ScanResult,
    pub selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Size,
    Age,
    Risk,
    Name,
}

#[derive(Debug)]
pub struct InteractiveSelector {
    items: Vec<SelectableItem>,
    list_state: ListState,
    sort_by: SortBy,
    show_help: bool,
}

impl InteractiveSelector {
    pub fn new(scan_results: Vec<ScanResult>) -> Self {
        let mut items: Vec<SelectableItem> = scan_results
            .into_iter()
            .map(|result| SelectableItem {
                scan_result: result,
                selected: false,
            })
            .collect();

        // Default sort by size (largest first)
        items.sort_by(|a, b| b.scan_result.size.cmp(&a.scan_result.size));

        let mut list_state = ListState::default();
        if !items.is_empty() {
            list_state.select(Some(0));
        }

        InteractiveSelector {
            items,
            list_state,
            sort_by: SortBy::Size,
            show_help: false,
        }
    }

    pub fn run(&mut self) -> io::Result<Vec<ScanResult>> {
        if self.items.is_empty() {
            return Ok(vec![]);
        }

        // Create cleanup guard to ensure terminal is restored even on panic
        let _guard = TerminalCleanupGuard::new();

        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_ui(&mut terminal);

        // Restore terminal (guard will also handle this if we panic)
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn run_ui(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<Vec<ScanResult>> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            if event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok(vec![]); // User cancelled
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(vec![]); // User cancelled with Ctrl+C
                        }
                        KeyCode::Enter => {
                            // Return selected items
                            return Ok(self.get_selected_items());
                        }
                        KeyCode::Char(' ') => {
                            self.toggle_current_item();
                        }
                        KeyCode::Char('a') => {
                            self.toggle_all_items();
                        }
                        KeyCode::Char('s') => {
                            self.cycle_sort();
                        }
                        KeyCode::Char('h') | KeyCode::Char('?') => {
                            self.show_help = !self.show_help;
                        }
                        KeyCode::Up => {
                            self.previous_item();
                        }
                        KeyCode::Down => {
                            self.next_item();
                        }
                        KeyCode::Home => {
                            if !self.items.is_empty() {
                                self.list_state.select(Some(0));
                            }
                        }
                        KeyCode::End => {
                            if !self.items.is_empty() {
                                self.list_state.select(Some(self.items.len() - 1));
                            }
                        }
                        KeyCode::PageUp => {
                            self.page_up();
                        }
                        KeyCode::PageDown => {
                            self.page_down();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&self, f: &mut Frame<CrosstermBackend<io::Stdout>>) {
        if self.show_help {
            self.draw_help(f);
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(5),    // File list
                Constraint::Length(3), // Footer
            ])
            .split(f.size());

        // Header
        self.draw_header(f, chunks[0]);

        // File list
        self.draw_file_list(f, chunks[1]);

        // Footer
        self.draw_footer(f, chunks[2]);
    }

    fn draw_header(&self, f: &mut Frame<CrosstermBackend<io::Stdout>>, area: tui::layout::Rect) {
        let selected_count = self.items.iter().filter(|item| item.selected).count();
        let total_size = self
            .items
            .iter()
            .filter(|item| item.selected)
            .map(|item| item.scan_result.size)
            .sum::<u64>();

        let size_str = super::utils::format_size(total_size);
        let sort_indicator = match self.sort_by {
            SortBy::Size => "Size ↓",
            SortBy::Age => "Age",
            SortBy::Risk => "Risk",
            SortBy::Name => "Name",
        };

        let header_text = format!(
            "Large Files - Selected: {}/{} ({}) - Sort: {} - Press 'h' for help",
            selected_count,
            self.items.len(),
            size_str,
            sort_indicator
        );

        let header = Paragraph::new(header_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Sweep Large File Cleanup"),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(header, area);
    }

    fn draw_file_list(&self, f: &mut Frame<CrosstermBackend<io::Stdout>>, area: tui::layout::Rect) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| {
                let checkbox = if item.selected { "☑" } else { "☐" };
                let risk_color = match item.scan_result.risk_level {
                    RiskLevel::Safe => Color::Green,
                    RiskLevel::Low => Color::Yellow,
                    RiskLevel::Medium => Color::Magenta,
                    RiskLevel::High => Color::Red,
                    RiskLevel::Critical => Color::LightRed,
                };

                let size_str = super::utils::format_size(item.scan_result.size);
                let risk_str = format!("{:?}", item.scan_result.risk_level);
                let path_str = item.scan_result.path.to_string_lossy();

                let line = Spans::from(vec![
                    Span::raw(format!("{} ", checkbox)),
                    Span::styled(
                        format!("{:>8} ", size_str),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(format!("{:>8} ", risk_str), Style::default().fg(risk_color)),
                    Span::raw(path_str),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Files"))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");

        f.render_stateful_widget(list, area, &mut self.list_state.clone());
    }

    fn draw_footer(&self, f: &mut Frame<CrosstermBackend<io::Stdout>>, area: tui::layout::Rect) {
        let footer_text =
            "Space: Toggle | Enter: Confirm | a: Toggle All | s: Sort | q/Esc: Cancel | h: Help";
        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);

        f.render_widget(footer, area);
    }

    fn draw_help(&self, f: &mut Frame<CrosstermBackend<io::Stdout>>) {
        let help_text = vec![
            "HELP - Large File Selection",
            "",
            "Navigation:",
            "  ↑/↓         Move selection up/down",
            "  Home/End    Go to first/last item",
            "  PgUp/PgDn   Move page up/down",
            "",
            "Selection:",
            "  Space       Toggle current item",
            "  a           Toggle all items",
            "",
            "Sorting:",
            "  s           Cycle sort order (Size → Age → Risk → Name)",
            "",
            "Actions:",
            "  Enter       Confirm selection and proceed",
            "  q/Esc       Cancel and exit",
            "  h/?         Toggle this help",
            "",
            "Risk Levels:",
            "  Safe        Green - Safe to delete",
            "  Low         Yellow - Low risk",
            "  Medium      Magenta - Medium risk",
            "  High        Red - High risk",
            "  Critical    Light Red - Protected files",
            "",
            "Press 'h' again to close help",
        ];

        let help_paragraph = Paragraph::new(
            help_text
                .into_iter()
                .map(|line| Spans::from(Span::raw(line)))
                .collect::<Vec<_>>(),
        )
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });

        let area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .margin(2)
            .split(f.size())[0];

        f.render_widget(Clear, area);
        f.render_widget(help_paragraph, area);
    }

    fn toggle_current_item(&mut self) {
        if let Some(index) = self.list_state.selected() {
            if index < self.items.len() {
                self.items[index].selected = !self.items[index].selected;
            }
        }
    }

    fn toggle_all_items(&mut self) {
        let all_selected = self.items.iter().all(|item| item.selected);
        for item in &mut self.items {
            item.selected = !all_selected;
        }
    }

    fn cycle_sort(&mut self) {
        self.sort_by = match self.sort_by {
            SortBy::Size => SortBy::Age,
            SortBy::Age => SortBy::Risk,
            SortBy::Risk => SortBy::Name,
            SortBy::Name => SortBy::Size,
        };
        self.sort_items();
    }

    fn sort_items(&mut self) {
        match self.sort_by {
            SortBy::Size => {
                self.items
                    .sort_by(|a, b| b.scan_result.size.cmp(&a.scan_result.size));
            }
            SortBy::Age => {
                // Sort by modification time (newer first) - this would require additional metadata
                // For now, we'll sort by description which contains age info
                self.items
                    .sort_by(|a, b| a.scan_result.description.cmp(&b.scan_result.description));
            }
            SortBy::Risk => {
                self.items.sort_by(|a, b| {
                    // Sort by risk level (critical first)
                    let risk_order = |level: &RiskLevel| match level {
                        RiskLevel::Critical => 0,
                        RiskLevel::High => 1,
                        RiskLevel::Medium => 2,
                        RiskLevel::Low => 3,
                        RiskLevel::Safe => 4,
                    };
                    risk_order(&a.scan_result.risk_level)
                        .cmp(&risk_order(&b.scan_result.risk_level))
                });
            }
            SortBy::Name => {
                self.items.sort_by(|a, b| {
                    a.scan_result
                        .path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("")
                        .cmp(
                            b.scan_result
                                .path
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or(""),
                        )
                });
            }
        }

        // Reset selection to first item after sorting
        if !self.items.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    fn get_selected_items(&self) -> Vec<ScanResult> {
        self.items
            .iter()
            .filter(|item| item.selected)
            .map(|item| item.scan_result.clone())
            .collect()
    }

    fn next_item(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous_item(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn page_up(&mut self) {
        let page_size = 10;
        let i = match self.list_state.selected() {
            Some(i) => i.saturating_sub(page_size),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn page_down(&mut self) {
        let page_size = 10;
        let i = match self.list_state.selected() {
            Some(i) => {
                let new_i = i + page_size;
                if new_i >= self.items.len() {
                    self.items.len() - 1
                } else {
                    new_i
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_scan_result(path: &str, size: u64, risk: RiskLevel) -> ScanResult {
        ScanResult {
            path: PathBuf::from(path),
            size,
            description: format!("{} | Test file", super::super::utils::format_size(size)),
            risk_level: risk,
        }
    }

    #[test]
    fn test_interactive_selector_creation() {
        let results = vec![
            create_test_scan_result("/test/large1.bin", 1000000, RiskLevel::Safe),
            create_test_scan_result("/test/large2.bin", 2000000, RiskLevel::Low),
        ];

        let selector = InteractiveSelector::new(results);
        assert_eq!(selector.items.len(), 2);
        assert_eq!(selector.sort_by, SortBy::Size);
        assert!(!selector.show_help);

        // Should be sorted by size (largest first)
        assert_eq!(selector.items[0].scan_result.size, 2000000);
        assert_eq!(selector.items[1].scan_result.size, 1000000);
    }

    #[test]
    fn test_toggle_selection() {
        let results = vec![create_test_scan_result(
            "/test/large1.bin",
            1000000,
            RiskLevel::Safe,
        )];

        let mut selector = InteractiveSelector::new(results);
        assert!(!selector.items[0].selected);

        selector.list_state.select(Some(0));
        selector.toggle_current_item();
        assert!(selector.items[0].selected);

        selector.toggle_current_item();
        assert!(!selector.items[0].selected);
    }

    #[test]
    fn test_toggle_all() {
        let results = vec![
            create_test_scan_result("/test/large1.bin", 1000000, RiskLevel::Safe),
            create_test_scan_result("/test/large2.bin", 2000000, RiskLevel::Low),
        ];

        let mut selector = InteractiveSelector::new(results);

        // Initially nothing selected
        assert!(!selector.items[0].selected);
        assert!(!selector.items[1].selected);

        // Toggle all - should select all
        selector.toggle_all_items();
        assert!(selector.items[0].selected);
        assert!(selector.items[1].selected);

        // Toggle all again - should deselect all
        selector.toggle_all_items();
        assert!(!selector.items[0].selected);
        assert!(!selector.items[1].selected);
    }

    #[test]
    fn test_sort_cycle() {
        let results = vec![
            create_test_scan_result("/test/small.bin", 1000, RiskLevel::High),
            create_test_scan_result("/test/large.bin", 2000000, RiskLevel::Safe),
        ];

        let mut selector = InteractiveSelector::new(results);
        assert_eq!(selector.sort_by, SortBy::Size);

        selector.cycle_sort();
        assert_eq!(selector.sort_by, SortBy::Age);

        selector.cycle_sort();
        assert_eq!(selector.sort_by, SortBy::Risk);

        selector.cycle_sort();
        assert_eq!(selector.sort_by, SortBy::Name);

        selector.cycle_sort();
        assert_eq!(selector.sort_by, SortBy::Size);
    }

    #[test]
    fn test_risk_level_sorting() {
        let results = vec![
            create_test_scan_result("/test/safe.bin", 1000, RiskLevel::Safe),
            create_test_scan_result("/test/critical.bin", 2000, RiskLevel::Critical),
            create_test_scan_result("/test/medium.bin", 3000, RiskLevel::Medium),
        ];

        let mut selector = InteractiveSelector::new(results);
        selector.sort_by = SortBy::Risk;
        selector.sort_items();

        // Should be sorted with critical first
        assert_eq!(
            selector.items[0].scan_result.risk_level,
            RiskLevel::Critical
        );
        assert_eq!(selector.items[1].scan_result.risk_level, RiskLevel::Medium);
        assert_eq!(selector.items[2].scan_result.risk_level, RiskLevel::Safe);
    }

    #[test]
    fn test_get_selected_items() {
        let results = vec![
            create_test_scan_result("/test/large1.bin", 1000000, RiskLevel::Safe),
            create_test_scan_result("/test/large2.bin", 2000000, RiskLevel::Low),
            create_test_scan_result("/test/large3.bin", 3000000, RiskLevel::Medium),
        ];

        let mut selector = InteractiveSelector::new(results);

        // Select first and third items
        selector.items[0].selected = true;
        selector.items[2].selected = true;

        let selected = selector.get_selected_items();
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].size, 3000000); // Sorted by size, so largest first
        assert_eq!(selected[1].size, 1000000);
    }

    #[test]
    fn test_navigation() {
        let results = vec![
            create_test_scan_result("/test/1.bin", 1000, RiskLevel::Safe),
            create_test_scan_result("/test/2.bin", 2000, RiskLevel::Safe),
            create_test_scan_result("/test/3.bin", 3000, RiskLevel::Safe),
        ];

        let mut selector = InteractiveSelector::new(results);

        // Should start at first item
        assert_eq!(selector.list_state.selected(), Some(0));

        // Move down
        selector.next_item();
        assert_eq!(selector.list_state.selected(), Some(1));

        // Move up
        selector.previous_item();
        assert_eq!(selector.list_state.selected(), Some(0));

        // Move up from first (should wrap to last)
        selector.previous_item();
        assert_eq!(selector.list_state.selected(), Some(2));

        // Move down from last (should wrap to first)
        selector.next_item();
        assert_eq!(selector.list_state.selected(), Some(0));
    }

    #[test]
    fn test_empty_results() {
        let selector = InteractiveSelector::new(vec![]);
        assert_eq!(selector.items.len(), 0);
        assert_eq!(selector.list_state.selected(), None);

        let selected = selector.get_selected_items();
        assert_eq!(selected.len(), 0);
    }
}
