# Technical Specification

This is the technical specification for the spec detailed in @.agent-os/specs/2025-09-14-large-file-detection/spec.md

## Technical Requirements

### Plugin Flag System Architecture

#### Command-Line Interface Extension
```rust
#[derive(StructOpt)]
pub struct Cli {
    // Existing flags
    #[structopt(short, long)]
    paths: Vec<PathBuf>,

    // Plugin activation flags
    #[structopt(long = "large-files")]
    enable_large_files: bool,

    #[structopt(long = "python")]
    enable_python: bool,

    #[structopt(long = "java")]
    enable_java: bool,

    // Global plugin options
    #[structopt(long = "older-than", value_name = "DAYS")]
    older_than_days: Option<u64>,

    // Large file specific options
    #[structopt(long = "size-threshold", default_value = "100MB")]
    size_threshold: String,

    #[structopt(long = "include-git-tracked")]
    include_git_tracked: bool,
}
```

#### Plugin Trait Extension
```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn is_enabled(&self, cli: &Cli) -> bool;
    fn configure(&mut self, cli: &Cli) -> Result<()>;
    fn apply_age_filter(&self, days: u64) -> Result<()>;
}

pub trait FeaturePlugin: Plugin {
    fn scan(&self, path: &Path) -> Result<Vec<ScanResult>>;
    fn interactive_select(&self, results: Vec<ScanResult>) -> Result<Vec<ScanResult>>;
    fn clean(&self, selected: Vec<ScanResult>) -> Result<CleanupReport>;
}
```

### Large File Detection Implementation

#### Core Scanner
```rust
pub struct LargeFilePlugin {
    size_threshold_bytes: u64,
    older_than_days: Option<u64>,
    include_git_tracked: bool,
    git_repos: HashMap<PathBuf, GitRepo>,
}

impl LargeFilePlugin {
    pub fn new() -> Self {
        Self {
            size_threshold_bytes: 100 * 1024 * 1024, // 100MB default
            older_than_days: None,
            include_git_tracked: false,
            git_repos: HashMap::new(),
        }
    }

    fn parse_size_string(&self, size: &str) -> Result<u64> {
        // Parse formats: "100MB", "1.5GB", "500K"
        // Support units: B, KB, MB, GB, TB
    }

    fn scan_directory(&self, path: &Path) -> Result<Vec<LargeFile>> {
        // Parallel scanning using rayon
        // Skip symbolic links
        // Handle permission errors gracefully
    }
}
```

#### File Analysis Structure
```rust
#[derive(Debug, Clone)]
pub struct LargeFile {
    pub path: PathBuf,
    pub size: u64,
    pub last_modified: SystemTime,
    pub last_accessed: SystemTime,
    pub is_git_tracked: bool,
    pub git_status: Option<GitFileStatus>,
    pub risk_level: RiskLevel,
    pub file_type: FileType,
}

#[derive(Debug, Clone)]
pub enum RiskLevel {
    Safe,      // Not tracked, old, clearly temporary
    Low,       // Old but might be useful
    Medium,    // Recent or matches test patterns
    High,      // Git tracked or very recent
    Critical,  // Never delete (in .gitignore as protected)
}

#[derive(Debug, Clone)]
pub enum FileType {
    TestData,
    Database,
    Archive,
    Media,
    Log,
    Binary,
    Unknown,
}
```

### Smart Filtering Engine

#### Multi-Factor Analysis
```rust
impl LargeFilePlugin {
    fn analyze_file(&self, file: &mut LargeFile) -> Result<()> {
        // Check git tracking status
        file.is_git_tracked = self.check_git_status(&file.path)?;

        // Analyze file type from extension and content
        file.file_type = self.detect_file_type(&file.path)?;

        // Calculate risk level
        file.risk_level = self.calculate_risk_level(file)?;

        Ok(())
    }

    fn calculate_risk_level(&self, file: &LargeFile) -> RiskLevel {
        // Critical: Git tracked (unless --include-git-tracked)
        if file.is_git_tracked && !self.include_git_tracked {
            return RiskLevel::Critical;
        }

        // High: Modified in last 7 days
        if file.last_modified > SystemTime::now() - Duration::from_days(7) {
            return RiskLevel::High;
        }

        // Medium: Matches test data patterns
        if self.is_test_data_pattern(&file.path) {
            return RiskLevel::Medium;
        }

        // Low: Old but recognizable file type
        if file.file_type != FileType::Unknown {
            return RiskLevel::Low;
        }

        RiskLevel::Safe
    }

    fn is_test_data_pattern(&self, path: &Path) -> bool {
        // Check for patterns like:
        // - test-data-*
        // - fixture*
        // - sample-*
        // - *.test.*
    }
}
```

#### Git Integration
```rust
use git2::{Repository, Status};

impl LargeFilePlugin {
    fn discover_git_repos(&mut self, root: &Path) -> Result<()> {
        // Find all .git directories
        // Cache repository handles
        // Build file status maps
    }

    fn check_git_status(&self, file_path: &Path) -> Result<bool> {
        // Find containing repository
        // Check if file is tracked
        // Return tracking status
    }
}
```

### Interactive Selection UI

#### Terminal Interface
```rust
use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
    ExecutableCommand,
};
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};

pub struct FileSelector {
    files: Vec<LargeFile>,
    selected: HashSet<usize>,
    list_state: ListState,
    sort_by: SortField,
    filter: FilterOptions,
}

impl FileSelector {
    pub fn run(&mut self) -> Result<Vec<LargeFile>> {
        // Setup terminal
        terminal::enable_raw_mode()?;

        // Event loop
        loop {
            self.draw()?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => self.previous(),
                    KeyCode::Down => self.next(),
                    KeyCode::Char(' ') => self.toggle_selection(),
                    KeyCode::Char('a') => self.select_all(),
                    KeyCode::Char('n') => self.select_none(),
                    KeyCode::Char('s') => self.cycle_sort(),
                    KeyCode::Char('f') => self.toggle_filter(),
                    KeyCode::Enter => break,
                    KeyCode::Esc | KeyCode::Char('q') => return Ok(vec![]),
                    _ => {}
                }
            }
        }

        // Return selected files
        Ok(self.get_selected_files())
    }

    fn draw(&self) -> Result<()> {
        // Render file list with:
        // - Checkbox indicators
        // - File paths (truncated if needed)
        // - Size (human readable)
        // - Age (days since modified)
        // - Risk level (color coded)
        // - Git status indicator
    }
}
```

#### Display Formatting
```rust
impl LargeFile {
    fn format_for_display(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.format_selection_indicator(),
            self.format_risk_indicator(),
            self.format_size(),
            self.format_age(),
            self.format_path()
        )
    }

    fn format_size(&self) -> String {
        // Human readable: 1.5GB, 500MB, etc.
    }

    fn format_risk_indicator(&self) -> String {
        match self.risk_level {
            RiskLevel::Safe => "✓",
            RiskLevel::Low => "◎",
            RiskLevel::Medium => "⚠",
            RiskLevel::High => "⚡",
            RiskLevel::Critical => "🔒",
        }
    }
}
```

### Age-Based Filtering

#### Time-Based Filtering
```rust
impl Plugin for LargeFilePlugin {
    fn apply_age_filter(&mut self, days: u64) -> Result<()> {
        self.older_than_days = Some(days);
        Ok(())
    }
}

impl LargeFilePlugin {
    fn should_include_by_age(&self, file: &LargeFile) -> bool {
        match self.older_than_days {
            Some(days) => {
                let age = SystemTime::now()
                    .duration_since(file.last_accessed)
                    .unwrap_or_default();
                age > Duration::from_secs(days * 24 * 60 * 60)
            }
            None => true,
        }
    }
}
```

### Performance Optimizations

#### Parallel Scanning
```rust
use rayon::prelude::*;
use crossbeam::channel;

impl LargeFilePlugin {
    fn scan_parallel(&self, roots: Vec<PathBuf>) -> Result<Vec<LargeFile>> {
        let (tx, rx) = channel::unbounded();

        roots.par_iter().for_each(|root| {
            WalkDir::new(root)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .for_each(|entry| {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.len() >= self.size_threshold_bytes {
                            // Send to channel for processing
                            tx.send(entry.path().to_path_buf()).ok();
                        }
                    }
                });
        });

        // Collect and analyze results
        let mut results = Vec::new();
        while let Ok(path) = rx.try_recv() {
            if let Ok(mut file) = self.analyze_path(&path) {
                results.push(file);
            }
        }

        Ok(results)
    }
}
```

### Error Handling

#### Graceful Degradation
```rust
impl LargeFilePlugin {
    fn handle_scan_error(&self, path: &Path, error: std::io::Error) {
        match error.kind() {
            std::io::ErrorKind::PermissionDenied => {
                // Log warning, continue scanning
            }
            std::io::ErrorKind::NotFound => {
                // File deleted during scan, ignore
            }
            _ => {
                // Log error, continue with other files
            }
        }
    }
}
```

## External Dependencies

### New Core Dependencies

- **tui** v0.19.x - Terminal UI framework for interactive selection
  - **Justification:** Required for building the interactive file selection interface

- **crossterm** v0.27.x - Cross-platform terminal manipulation
  - **Justification:** Terminal control for the TUI, works on Windows/Mac/Linux

- **git2** v0.18.x - Git repository interaction
  - **Justification:** Check if files are tracked in git for safety filtering

- **humansize** v2.1.x - Human-readable file sizes
  - **Justification:** Display file sizes in user-friendly format (MB, GB, etc.)

- **chrono** v0.4.x - Date and time handling
  - **Justification:** Parse and display file ages and time-based filtering

### Optional Dependencies

- **indicatif** v0.17.x - Progress bars for scanning
  - **Justification:** Show progress during long directory scans

- **fuzzy-matcher** v0.3.x - Fuzzy search in file list
  - **Justification:** Allow users to search/filter files in the interactive UI