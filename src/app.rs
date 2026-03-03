use eframe::egui;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver};
use std::fs;

use crate::document_processor::{DocumentProcessor, ExtractionOptions};
use crate::ai_extractor::AIExtractor;
use crate::csv_exporter::{CSVExporter, ExportFormat};

/// A field definition with optional AI-enhanced extraction prompt
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FieldDefinition {
    /// The field name (used as CSV column header)
    pub name: String,
    /// AI-generated extraction prompt (None if not yet enhanced or is a default field)
    pub extraction_prompt: Option<String>,
    /// Whether this is a built-in default field (uses keyword-based extraction)
    pub is_default: bool,
    /// Whether prompt enhancement is currently in progress
    #[serde(default)]
    pub is_enhancing: bool,
}

impl FieldDefinition {
    /// Create a new default field (built-in fields that use keyword-based extraction)
    pub fn new_default(name: String) -> Self {
        Self {
            name,
            extraction_prompt: None,
            is_default: true,
            is_enhancing: false,
        }
    }

    /// Create a new custom field (user-added fields that need AI enhancement)
    pub fn new_custom(name: String) -> Self {
        Self {
            name,
            extraction_prompt: None,
            is_default: false,
            is_enhancing: false,
        }
    }

    /// Get the field name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Extract just the field names from a Vec<FieldDefinition>
fn get_field_names(fields: &[FieldDefinition]) -> Vec<String> {
    fields.iter().map(|f| f.name.clone()).collect()
}

/// Extract custom prompts from FieldDefinitions (field_name -> prompt)
fn get_custom_prompts(fields: &[FieldDefinition]) -> std::collections::HashMap<String, String> {
    fields.iter()
        .filter_map(|f| {
            f.extraction_prompt.as_ref().map(|prompt| (f.name.clone(), prompt.clone()))
        })
        .collect()
}

/// List of default field names that use keyword-based extraction
const DEFAULT_EXTRACTION_FIELDS: &[&str] = &[
    "Date",
    "Vendor/Store Name",
    "Seller",
    "Total Amount",
    "Tax Amount",
    "Payment Method",
    "item",
    "item summary",
];

const DEFAULT_ITEM_FIELDS: &[&str] = &[
    "Item Name",
    "Quantity",
    "Unit Price",
    "Line Total",
];

/// Check if a field name is a default field
fn is_default_field(name: &str, default_list: &[&str]) -> bool {
    default_list.iter().any(|&d| d.eq_ignore_ascii_case(name))
}

/// Get the path to the extraction fields config file
fn get_fields_config_path() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    let app_dir = config_dir.join("receipt_extractor");
    Some(app_dir.join("extraction_fields.json"))
}

/// Load saved extraction fields from config file
/// Handles migration from old Vec<String> format to new Vec<FieldDefinition> format
fn load_extraction_fields() -> Option<Vec<FieldDefinition>> {
    let path = get_fields_config_path()?;
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;

    // Try new format first
    if let Ok(fields) = serde_json::from_str::<Vec<FieldDefinition>>(&content) {
        return Some(fields);
    }

    // Fallback to old format and migrate
    if let Ok(old_fields) = serde_json::from_str::<Vec<String>>(&content) {
        return Some(old_fields.into_iter()
            .map(|name| {
                if is_default_field(&name, DEFAULT_EXTRACTION_FIELDS) {
                    FieldDefinition::new_default(name)
                } else {
                    FieldDefinition::new_custom(name)
                }
            })
            .collect());
    }

    None
}

/// Save extraction fields to config file
fn save_extraction_fields(fields: &[FieldDefinition]) {
    if let Some(path) = get_fields_config_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(fields) {
            let _ = fs::write(&path, json);
        }
    }
}

/// Get the path to the item fields config file
fn get_item_fields_config_path() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    let app_dir = config_dir.join("receipt_extractor");
    Some(app_dir.join("item_fields.json"))
}

/// Load saved item fields from config file
/// Handles migration from old Vec<String> format to new Vec<FieldDefinition> format
fn load_item_fields() -> Option<Vec<FieldDefinition>> {
    let path = get_item_fields_config_path()?;
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;

    // Try new format first
    if let Ok(fields) = serde_json::from_str::<Vec<FieldDefinition>>(&content) {
        return Some(fields);
    }

    // Fallback to old format and migrate
    if let Ok(old_fields) = serde_json::from_str::<Vec<String>>(&content) {
        return Some(old_fields.into_iter()
            .map(|name| {
                if is_default_field(&name, DEFAULT_ITEM_FIELDS) {
                    FieldDefinition::new_default(name)
                } else {
                    FieldDefinition::new_custom(name)
                }
            })
            .collect());
    }

    None
}

/// Save item fields to config file
fn save_item_fields(fields: &[FieldDefinition]) {
    if let Some(path) = get_item_fields_config_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(fields) {
            let _ = fs::write(&path, json);
        }
    }
}

/// Settings structure for API keys
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct ApiSettings {
    groq_api_key: String,
    openai_api_key: String,
    claude_api_key: String,
    gemini_api_key: String,
    openrouter_api_key: String,
    #[serde(default)]
    use_latest_models: bool,  // Toggle between stable and latest model versions
    #[serde(default)]
    selected_provider: Option<AIProvider>,  // Remember which AI provider was selected
    #[serde(default = "default_show_tips")]
    show_tips_on_startup: bool,  // Show tip widget on startup
    #[serde(default = "default_show_api_keys")]
    show_api_keys_section: bool,  // Show/hide API keys section
    #[serde(default = "default_extraction_mode")]
    extraction_mode: ExtractionMode,  // General or Detailed extraction mode
    #[serde(default)]
    enable_debug_logging: bool,  // Enable debug logging to file
    #[serde(default)]
    export_format: ExportFormat,  // Export file format (CSV, Excel, JSON)
    #[serde(default)]
    onboarding_completed: bool,  // Track if user has completed onboarding
    #[serde(default)]
    last_open_directory: Option<String>,  // Remember last directory for file dialogs
    #[serde(default)]
    last_save_directory: Option<String>,  // Remember last directory for save dialogs
    #[serde(default = "default_ollama_url")]
    ollama_url: String,  // Local LLM base URL (Ollama/LM Studio/Jan)
    #[serde(default)]
    local_model_name: String,  // Local LLM model name (e.g. "llava")
}

fn default_show_tips() -> bool {
    true  // Tips enabled by default
}

fn default_show_api_keys() -> bool {
    true  // API keys section expanded by default
}

fn default_extraction_mode() -> ExtractionMode {
    ExtractionMode::General  // General extraction by default
}

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

/// Extraction mode - either General (summary per receipt) or Detailed (items per receipt)
#[derive(Default, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
enum ExtractionMode {
    #[default]
    General,   // One row per receipt with summary fields
    Detailed,  // One row per item with itemized data
}

/// Get a list of helpful tips for users
fn get_tips() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Quick Start", "Click 'Settings' to add your API key, then select files and click 'Extract Data'!"),
        ("Free Options", "Groq and Gemini offer free tiers! Perfect for testing before committing to paid services."),
        ("Auto-Save", "Set an output path and the app will auto-save after each extraction to prevent data loss."),
        ("Stable vs Latest", "Use 'Stable' models for consistent results, or 'Latest' for cutting-edge performance."),
        ("Rate Limits", "If you hit rate limits, the app will automatically wait and retry - no need to restart!"),
        ("Image Quality", "For best results, use clear, well-lit receipt images with all text visible."),
        ("Batch Processing", "You can select and process multiple receipts at once to save time."),
        ("Custom Fields", "Add any extraction fields you need: Date, Merchant, Total, Tax, Items, etc."),
        ("Review Results", "Always review incomplete extractions - you can manually fill in missing data."),
        ("Need Help?", "Click the 'Help' button in the header for detailed setup instructions and API key guides."),
    ]
}

/// Get the path to the API settings config file
fn get_api_settings_path() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    let app_dir = config_dir.join("receipt_extractor");
    Some(app_dir.join("api_settings.json"))
}

/// Load saved API settings from config file
fn load_api_settings() -> ApiSettings {
    if let Some(path) = get_api_settings_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str(&content) {
                    return settings;
                }
            }
        }
    }
    ApiSettings::default()
}

/// Save API settings to config file
fn save_api_settings(settings: &ApiSettings) {
    if let Some(path) = get_api_settings_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(settings) {
            let _ = fs::write(&path, json);
        }
    }
}


#[derive(Debug)]
enum ProcessingMessage {
    Progress(usize, usize, String), // current, total, current_file_name
    ProcessingComplete,             // Processing done, ready to save
    Error(String),
    RateLimitCountdown(u64),        // Seconds remaining until retry
}

#[derive(Default, PartialEq)]
enum AppState {
    #[default]
    Idle,
    Processing,
    ReadyToSave,        // Processing complete, waiting for save location
    Completed,
    Error,
    Settings,           // Settings page
    Onboarding,         // First-time user onboarding wizard
}

/// Onboarding wizard steps
#[derive(Default, PartialEq, Clone, Copy)]
enum OnboardingStep {
    #[default]
    Welcome,            // Welcome message
    SelectProvider,     // Choose AI provider
    EnterApiKey,        // Enter API key
    TestApiKey,         // Test the API key
    Complete,           // Onboarding complete
}

#[derive(Default, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
enum AIProvider {
    #[default]
    Groq,
    ChatGPT,
    Claude,
    Gemini,
    OpenRouter,
    LocalLLM,
}

impl AIProvider {
    fn name(&self) -> &str {
        match self {
            AIProvider::Groq => "Groq (llama-3.2-90b-vision)",
            AIProvider::ChatGPT => "OpenAI (gpt-4o-mini)",
            AIProvider::Claude => "Anthropic Claude (claude-3-5-sonnet)",
            AIProvider::Gemini => "Google Gemini (gemini-2.5-flash)",
            AIProvider::OpenRouter => "OpenRouter (llama-3.2-90b-vision)",
            AIProvider::LocalLLM => "Local LLM (Ollama / LM Studio / Jan)",
        }
    }

    fn all() -> Vec<AIProvider> {
        vec![
            AIProvider::Groq,
            AIProvider::ChatGPT,
            AIProvider::Claude,
            AIProvider::Gemini,
            AIProvider::OpenRouter,
            AIProvider::LocalLLM,
        ]
    }

    fn description(&self) -> &str {
        match self {
            AIProvider::Groq => "Fast and cost-effective vision model using Llama 3.2 90B",
            AIProvider::ChatGPT => "OpenAI's GPT-4o-mini - reliable and accurate",
            AIProvider::Claude => "Anthropic's Claude 3.5 Sonnet - highly capable vision model",
            AIProvider::Gemini => "Google's Gemini 1.5 Flash - fast and efficient",
            AIProvider::OpenRouter => "Access multiple models through OpenRouter",
            AIProvider::LocalLLM => "Run models locally with Ollama, LM Studio, or Jan - free & private",
        }
    }
}

/// Tracks receipts with missing critical fields
#[derive(Debug, Clone)]
pub struct IncompleteExtraction {
    pub file_name: String,
    pub missing_fields: Vec<String>,
    /// Optional error reason explaining why extraction failed
    pub error_reason: Option<String>,
}

pub struct ReceiptExtractorApp {
    selected_files: Vec<PathBuf>,
    extraction_fields: Vec<FieldDefinition>,
    new_field_input: String,
    ai_provider: AIProvider,
    state: AppState,
    progress_message: String,
    error_message: String,
    output_path: Option<PathBuf>,
    processor: Arc<Mutex<Option<DocumentProcessor>>>,
    progress_rx: Option<Receiver<ProcessingMessage>>,
    current_progress: usize,
    total_progress: usize,

    // Incomplete extraction tracking
    incomplete_extractions: Vec<IncompleteExtraction>,

    // CSV output path (always visible in UI)
    csv_output_path: String,
    csv_save_error: Option<String>,
    // Auto-save is always enabled - saves after each successful extraction when path is set

    // Rate limit tracking
    rate_limit_warning: Option<String>,  // Warning message if rate limited
    rate_limit_countdown: Option<u64>,   // Seconds remaining until retry

    // Settings page - API Keys
    groq_api_key: String,
    openai_api_key: String,
    claude_api_key: String,
    gemini_api_key: String,
    openrouter_api_key: String,
    use_latest_models: bool,  // Toggle between stable and latest model versions

    // Local LLM settings (Ollama, LM Studio, Jan, etc.)
    ollama_url: String,        // Base URL for local LLM server
    local_model_name: String,  // Model name to use (e.g. "llava")

    // API key validation
    validating_api_key: bool,
    api_key_validation_result: Option<String>, // Success or error message
    api_validation_rx: Option<Receiver<Result<String, String>>>,

    // Tokio runtime for async operations
    runtime: tokio::runtime::Runtime,

    // File dialog state
    file_dialog_rx: Option<Receiver<FileDialogResult>>,

    // Tip widget
    show_tips_on_startup: bool,  // User preference for showing tips
    show_tip_widget: bool,       // Current state of tip widget visibility
    current_tip_index: usize,    // Which tip to show

    // Help modal
    show_help_modal: bool,       // Show the Getting Started Guide modal
    show_about_modal: bool,      // Show the About dialog

    // Settings UI state
    show_api_keys_section: bool, // Show/hide API keys section in Settings

    // Extraction mode (General or Detailed)
    extraction_mode: ExtractionMode,   // Current extraction mode
    item_fields: Vec<FieldDefinition>, // Fields for detailed item extraction
    new_item_field_input: String,      // Input for adding new item fields

    // Debug logging
    enable_debug_logging: bool,        // Enable debug logging to file

    // Prompt enhancement for custom fields
    enhancement_rx: Option<Receiver<(usize, Option<String>, bool)>>, // (field_index, enhanced_prompt, is_item_field)

    // Export format
    export_format: ExportFormat,       // Export file format (CSV, Excel, JSON)

    // Onboarding
    onboarding_completed: bool,        // Track if user has completed onboarding
    onboarding_step: OnboardingStep,   // Current step in onboarding wizard

    // Recent directories
    last_open_directory: Option<PathBuf>,  // Remember last directory for file dialogs
    last_save_directory: Option<PathBuf>,  // Remember last directory for save dialogs

    // Collapsible sections state
    documents_section_open: bool,      // Documents section collapsed state
    fields_section_open: bool,         // Extraction Fields section collapsed state

    // Drag-and-drop state for field reordering
    dragging_field_idx: Option<usize>,       // Index of field being dragged (extraction fields)
    dragging_field_text: Option<String>,     // Text of field being dragged (for ghost)
    dragging_item_field_idx: Option<usize>,  // Index of field being dragged (item fields)
    dragging_item_field_text: Option<String>, // Text of item field being dragged (for ghost)
    // Hover timing for auto-insert after 1.5 seconds
    drag_hover_target: Option<usize>,        // Current hover target index (extraction fields)
    drag_hover_start: Option<std::time::Instant>, // When hover started
    item_drag_hover_target: Option<usize>,   // Current hover target index (item fields)
    item_drag_hover_start: Option<std::time::Instant>, // When hover started
    // Animation state for dropped items
    field_drop_animation: Option<(usize, usize, std::time::Instant)>, // (from_idx, to_idx, start_time)
    item_field_drop_animation: Option<(usize, usize, std::time::Instant)>, // (from_idx, to_idx, start_time)

    // Toast notifications
    toast_message: Option<(String, std::time::Instant, ToastType)>, // (message, start_time, type)

    // Save button animation (pulse effect after extraction completes)
    save_button_pulse_start: Option<std::time::Instant>,

}

/// Toast notification type
#[derive(Clone, PartialEq)]
enum ToastType {
    Success,
    Error,
    Info,
}

#[derive(Debug)]
enum FileDialogResult {
    FilesToAdd(Vec<PathBuf>),
    CsvPathSelected(PathBuf),
}

impl ReceiptExtractorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create tokio runtime for async operations
        let runtime = tokio::runtime::Runtime::new()
            .expect("Failed to create tokio runtime");

        // Load saved extraction fields or use defaults
        let extraction_fields = load_extraction_fields().unwrap_or_else(|| {
            DEFAULT_EXTRACTION_FIELDS.iter()
                .map(|&name| FieldDefinition::new_default(name.to_string()))
                .collect()
        });

        // Load saved item fields or use defaults for detailed extraction
        let item_fields = load_item_fields().unwrap_or_else(|| {
            DEFAULT_ITEM_FIELDS.iter()
                .map(|&name| FieldDefinition::new_default(name.to_string()))
                .collect()
        });

        // Load saved API settings
        let api_settings = load_api_settings();

        // Initialize debug logging state from saved settings
        crate::debug_log::set_debug_enabled(api_settings.enable_debug_logging);

        // Determine initial state - show onboarding if not completed
        let initial_state = if api_settings.onboarding_completed {
            AppState::Idle
        } else {
            AppState::Onboarding
        };

        let mut app = Self {
            selected_files: Vec::new(),
            extraction_fields,
            new_field_input: String::new(),
            ai_provider: api_settings.selected_provider.unwrap_or(AIProvider::Groq),
            state: initial_state,
            progress_message: String::new(),
            error_message: String::new(),
            output_path: None,
            processor: Arc::new(Mutex::new(None)),
            progress_rx: None,
            current_progress: 0,
            total_progress: 0,
            incomplete_extractions: Vec::new(),
            csv_output_path: String::new(),
            csv_save_error: None,
            // Auto-save is always enabled
            rate_limit_warning: None,
            rate_limit_countdown: None,
            groq_api_key: api_settings.groq_api_key,
            openai_api_key: api_settings.openai_api_key,
            claude_api_key: api_settings.claude_api_key,
            gemini_api_key: api_settings.gemini_api_key,
            openrouter_api_key: api_settings.openrouter_api_key,
            use_latest_models: api_settings.use_latest_models,
            ollama_url: api_settings.ollama_url,
            local_model_name: api_settings.local_model_name,
            validating_api_key: false,
            api_key_validation_result: None,
            api_validation_rx: None,
            runtime,
            file_dialog_rx: None,
            show_tips_on_startup: api_settings.show_tips_on_startup,
            show_tip_widget: api_settings.show_tips_on_startup && api_settings.onboarding_completed,  // Only show tips if onboarding done
            current_tip_index: (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() % 10) as usize,  // Random tip based on current time
            show_help_modal: false,  // Don't show help modal on startup
            show_about_modal: false,  // Don't show about modal on startup
            show_api_keys_section: api_settings.show_api_keys_section,  // Load saved state
            extraction_mode: api_settings.extraction_mode,
            item_fields,
            new_item_field_input: String::new(),
            enable_debug_logging: api_settings.enable_debug_logging,
            export_format: api_settings.export_format,
            onboarding_completed: api_settings.onboarding_completed,
            onboarding_step: OnboardingStep::Welcome,
            last_open_directory: api_settings.last_open_directory.map(PathBuf::from),
            last_save_directory: api_settings.last_save_directory.map(PathBuf::from),
            documents_section_open: true,   // Open by default
            fields_section_open: true,      // Open by default
            dragging_field_idx: None,       // No field being dragged
            dragging_field_text: None,      // No field text for ghost
            dragging_item_field_idx: None,  // No item field being dragged
            dragging_item_field_text: None, // No item field text for ghost
            drag_hover_target: None,        // No hover target
            drag_hover_start: None,         // No hover timing
            item_drag_hover_target: None,   // No hover target (item fields)
            item_drag_hover_start: None,    // No hover timing (item fields)
            field_drop_animation: None,     // No drop animation
            item_field_drop_animation: None, // No drop animation (item fields)
            toast_message: None,            // No toast notification
            save_button_pulse_start: None,  // No save button pulse
            enhancement_rx: None,           // No prompt enhancement in progress
        };

        app
    }

    fn save_api_keys(&self) {
        let settings = ApiSettings {
            groq_api_key: self.groq_api_key.clone(),
            openai_api_key: self.openai_api_key.clone(),
            claude_api_key: self.claude_api_key.clone(),
            gemini_api_key: self.gemini_api_key.clone(),
            openrouter_api_key: self.openrouter_api_key.clone(),
            use_latest_models: self.use_latest_models,
            selected_provider: Some(self.ai_provider),
            show_tips_on_startup: self.show_tips_on_startup,
            show_api_keys_section: self.show_api_keys_section,
            extraction_mode: self.extraction_mode,
            enable_debug_logging: self.enable_debug_logging,
            export_format: self.export_format,
            onboarding_completed: self.onboarding_completed,
            last_open_directory: self.last_open_directory.as_ref().map(|p| p.to_string_lossy().to_string()),
            last_save_directory: self.last_save_directory.as_ref().map(|p| p.to_string_lossy().to_string()),
            ollama_url: self.ollama_url.clone(),
            local_model_name: self.local_model_name.clone(),
        };
        save_api_settings(&settings);
    }

    fn add_more_files(&mut self) {
        let (tx, rx) = channel();
        self.file_dialog_rx = Some(rx);

        // Clone the last directory for the async task
        let last_dir = self.last_open_directory.clone();

        // Spawn async task to show dialog without blocking UI
        self.runtime.spawn(async move {
            let mut dialog = rfd::AsyncFileDialog::new()
                .add_filter("Documents", &["pdf", "png", "jpg", "jpeg", "txt", "html", "htm"])
                .set_title("Add More Documents");

            // Set starting directory if we have one
            if let Some(dir) = last_dir {
                dialog = dialog.set_directory(&dir);
            }

            if let Some(file_handles) = dialog.pick_files().await {
                let files: Vec<PathBuf> = file_handles.iter().map(|f| f.path().to_path_buf()).collect();
                let _ = tx.send(FileDialogResult::FilesToAdd(files));
            }
        });
    }

    fn save_csv(&mut self) {
        // Clear any previous save error
        self.csv_save_error = None;

        // Use the pre-set path if available, otherwise show dialog
        let save_path = if !self.csv_output_path.is_empty() {
            let path = PathBuf::from(&self.csv_output_path);
            // Ensure it has .csv extension
            if path.extension().map_or(true, |ext| ext != "csv") {
                PathBuf::from(format!("{}.csv", self.csv_output_path))
            } else {
                path
            }
        } else {
            // Show save dialog if no path set - use appropriate extension for format
            let (filter_name, filter_ext, default_name) = match self.export_format {
                ExportFormat::Csv => ("CSV", "csv", "receipts_export.csv"),
                ExportFormat::Excel => ("Excel", "xlsx", "receipts_export.xlsx"),
                ExportFormat::Json => ("JSON", "json", "receipts_export.json"),
            };
            match rfd::FileDialog::new()
                .add_filter(filter_name, &[filter_ext])
                .set_file_name(default_name)
                .set_title("Save Receipt Data")
                .save_file()
            {
                Some(path) => {
                    // Update the path field for future saves
                    self.csv_output_path = path.to_string_lossy().to_string();
                    path
                }
                None => {
                    // User cancelled
                    return;
                }
            }
        };

        // Get processor results and save using selected format
        let enable_general = self.extraction_mode == ExtractionMode::General;
        let enable_detailed = self.extraction_mode == ExtractionMode::Detailed;

        // Store result info for toast after releasing the lock
        let mut toast_info: Option<(String, ToastType)> = None;

        {
            let proc_guard = self.processor.lock().unwrap();
            if let Some(proc) = proc_guard.as_ref() {
                let field_names = get_field_names(&self.extraction_fields);
                let item_field_names = get_field_names(&self.item_fields);
                match CSVExporter::export_with_format(
                    proc.get_results(),
                    &field_names,
                    &item_field_names,
                    enable_general,
                    enable_detailed,
                    save_path.clone(),
                    self.export_format,
                ) {
                    Ok(result) => {
                        // Build success message showing which files were created
                        let mut created_files = Vec::new();
                        if let Some(summary_path) = &result.summary_path {
                            println!("{} exported to: {:?}", self.export_format.name(), summary_path);
                            created_files.push(format!("Summary: {}", summary_path.display()));
                        }
                        if let Some(items_path) = &result.items_path {
                            println!("{} Items exported to: {:?}", self.export_format.name(), items_path);
                            created_files.push(format!("Items: {}", items_path.display()));
                        }

                        // Store the first available path for display
                        self.output_path = result.summary_path.or(result.items_path);
                        self.csv_save_error = None;
                        self.state = AppState::Completed;

                        // Prepare success toast
                        let count = proc.get_results().len();
                        toast_info = Some((format!("Saved {} receipt(s) successfully!", count), ToastType::Success));
                    }
                    Err(e) => {
                        eprintln!("Error exporting: {}", e);
                        // Show error but DON'T change state - keep save button visible
                        self.csv_save_error = Some(format!("Failed to save: {}. Close the file if it's open and try again.", e));
                        // Prepare error toast
                        toast_info = Some(("Failed to save file".to_string(), ToastType::Error));
                    }
                }
            }
        } // Lock released here

        // Show toast after lock is released
        if let Some((message, toast_type)) = toast_info {
            self.show_toast(&message, toast_type);
        }
    }

    fn browse_csv_path(&mut self) {
        let (tx, rx) = channel();
        self.file_dialog_rx = Some(rx);

        // Get export format info for the dialog
        let (filter_name, filter_ext, default_name) = match self.export_format {
            ExportFormat::Csv => ("CSV", "csv", "receipts_export.csv"),
            ExportFormat::Excel => ("Excel", "xlsx", "receipts_export.xlsx"),
            ExportFormat::Json => ("JSON", "json", "receipts_export.json"),
        };
        let filter_name = filter_name.to_string();
        let filter_ext = filter_ext.to_string();
        let default_name = default_name.to_string();

        // Clone the last directory for the async task
        let last_dir = self.last_save_directory.clone();

        // Spawn async task to show dialog without blocking UI
        self.runtime.spawn(async move {
            let mut dialog = rfd::AsyncFileDialog::new()
                .add_filter(&filter_name, &[&filter_ext])
                .set_file_name(&default_name)
                .set_title("Choose Output Location");

            // Set starting directory if we have one
            if let Some(dir) = last_dir {
                dialog = dialog.set_directory(&dir);
            }

            if let Some(file_handle) = dialog.save_file().await {
                let path = file_handle.path().to_path_buf();
                let _ = tx.send(FileDialogResult::CsvPathSelected(path));
            }
        });
    }

    fn show_onboarding_wizard(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.add_space(40.0);

        // Center the content
        ui.vertical_centered(|ui| {
            // Progress indicator
            let step_num = match self.onboarding_step {
                OnboardingStep::Welcome => 1,
                OnboardingStep::SelectProvider => 2,
                OnboardingStep::EnterApiKey => 3,
                OnboardingStep::TestApiKey => 4,
                OnboardingStep::Complete => 5,
            };
            ui.label(egui::RichText::new(format!("Step {} of 4", step_num.min(4)))
                .color(egui::Color32::GRAY)
                .size(12.0));

            ui.add_space(20.0);

            match self.onboarding_step {
                OnboardingStep::Welcome => {
                    // Welcome screen
                    ui.label(egui::RichText::new("★").size(64.0).color(egui::Color32::from_rgb(251, 191, 36)));
                    ui.add_space(20.0);
                    ui.label(egui::RichText::new("Welcome to Receipt Data Extractor!")
                        .strong()
                        .size(28.0));
                    ui.add_space(15.0);
                    ui.label(egui::RichText::new("Extract data from receipts using AI vision models.")
                        .size(16.0)
                        .color(egui::Color32::LIGHT_GRAY));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("This quick setup will help you get started in just a few steps.")
                        .size(14.0)
                        .color(egui::Color32::GRAY));

                    ui.add_space(40.0);

                    if ui.add(egui::Button::new(egui::RichText::new("Get Started →").size(16.0))
                        .fill(egui::Color32::from_rgb(59, 130, 246))
                        .min_size(egui::vec2(200.0, 45.0)))
                        .clicked() {
                        self.onboarding_step = OnboardingStep::SelectProvider;
                    }

                    ui.add_space(20.0);
                    if ui.small_button("Skip setup (I'll configure later)")
                        .on_hover_text("You can always access settings from the main screen")
                        .clicked() {
                        self.onboarding_completed = true;
                        self.state = AppState::Idle;
                        self.save_api_keys();
                    }
                }

                OnboardingStep::SelectProvider => {
                    // Provider selection
                    ui.label(egui::RichText::new("⚙️").size(36.0).color(egui::Color32::from_rgb(59, 130, 246)));
                    ui.add_space(15.0);
                    ui.label(egui::RichText::new("Choose Your AI Provider")
                        .strong()
                        .size(24.0));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Select which AI service you'd like to use for receipt extraction.")
                        .size(14.0)
                        .color(egui::Color32::GRAY));

                    ui.add_space(30.0);

                    // Provider cards
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(30, 41, 59))
                        .rounding(egui::Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(20.0))
                        .show(ui, |ui| {
                            ui.set_width(400.0);

                            for provider in AIProvider::all() {
                                let is_selected = self.ai_provider == provider;
                                let bg_color = if is_selected {
                                    egui::Color32::from_rgb(59, 130, 246)
                                } else {
                                    egui::Color32::from_rgb(51, 65, 85)
                                };

                                egui::Frame::none()
                                    .fill(bg_color)
                                    .rounding(egui::Rounding::same(8.0))
                                    .inner_margin(egui::Margin::same(12.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.radio_value(&mut self.ai_provider, provider, "");
                                            ui.vertical(|ui| {
                                                ui.label(egui::RichText::new(provider.name())
                                                    .strong()
                                                    .color(egui::Color32::WHITE));
                                                ui.label(egui::RichText::new(provider.description())
                                                    .size(11.0)
                                                    .color(if is_selected { egui::Color32::from_rgb(191, 219, 254) } else { egui::Color32::GRAY }));

                                                // Show free tier info
                                                let free_info = match provider {
                                                    AIProvider::Groq => Some("★ Free tier: 30 requests/min"),
                                                    AIProvider::Gemini => Some("★ Free tier: 15 requests/min"),
                                                    _ => None,
                                                };
                                                if let Some(info) = free_info {
                                                    ui.label(egui::RichText::new(info)
                                                        .size(10.0)
                                                        .color(egui::Color32::from_rgb(134, 239, 172)));
                                                }
                                            });
                                        });
                                    });
                                ui.add_space(8.0);
                            }
                        });

                    ui.add_space(30.0);

                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(egui::RichText::new("← Back").size(13.0))
                            .min_size(egui::vec2(80.0, 35.0)))
                            .clicked() {
                            self.onboarding_step = OnboardingStep::Welcome;
                        }
                        ui.add_space(20.0);
                        if ui.add(egui::Button::new(egui::RichText::new("Continue →").size(14.0))
                            .fill(egui::Color32::from_rgb(59, 130, 246))
                            .min_size(egui::vec2(120.0, 35.0)))
                            .clicked() {
                            self.onboarding_step = OnboardingStep::EnterApiKey;
                        }
                    });
                }

                OnboardingStep::EnterApiKey => {
                    let is_local_llm = self.ai_provider == AIProvider::LocalLLM;

                    // Header
                    ui.label(egui::RichText::new(if is_local_llm { "🖥" } else { "🔑" }).size(36.0).color(egui::Color32::from_rgb(251, 191, 36)));
                    ui.add_space(15.0);
                    ui.label(egui::RichText::new(if is_local_llm { "Configure Local LLM" } else { "Enter Your API Key" })
                        .strong()
                        .size(24.0));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(if is_local_llm {
                        "Enter the URL of your local LLM server (Ollama, LM Studio, Jan, etc.)".to_string()
                    } else {
                        format!("Enter your {} API key to continue.", self.ai_provider.name())
                    })
                        .size(14.0)
                        .color(egui::Color32::GRAY));

                    ui.add_space(30.0);

                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(30, 41, 59))
                        .rounding(egui::Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(20.0))
                        .show(ui, |ui| {
                            ui.set_width(450.0);

                            if is_local_llm {
                                // Local LLM: show URL and model name inputs
                                ui.label(egui::RichText::new("Server URL:").strong());
                                ui.add_space(8.0);
                                ui.add(egui::TextEdit::singleline(&mut self.ollama_url)
                                    .hint_text("http://localhost:11434")
                                    .desired_width(400.0));

                                ui.add_space(10.0);
                                ui.label(egui::RichText::new("Model Name:").strong());
                                ui.add_space(8.0);
                                ui.add(egui::TextEdit::singleline(&mut self.local_model_name)
                                    .hint_text("llava")
                                    .desired_width(400.0));
                                ui.label(egui::RichText::new("Use a vision-capable model (e.g. llava, llama3.2-vision)")
                                    .color(egui::Color32::GRAY).size(11.0));

                                ui.add_space(15.0);
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Don't have Ollama?").color(egui::Color32::GRAY).size(12.0));
                                    if ui.link("Download here →").clicked() {
                                        let _ = opener::open("https://ollama.com");
                                    }
                                });
                            } else {
                                // Cloud provider: show API key input
                                let (api_key, hint, get_key_url) = match self.ai_provider {
                                    AIProvider::Groq => (&mut self.groq_api_key, "gsk_...", "https://console.groq.com/keys"),
                                    AIProvider::ChatGPT => (&mut self.openai_api_key, "sk-...", "https://platform.openai.com/api-keys"),
                                    AIProvider::Claude => (&mut self.claude_api_key, "sk-ant-...", "https://console.anthropic.com/settings/keys"),
                                    AIProvider::Gemini => (&mut self.gemini_api_key, "AIza...", "https://aistudio.google.com/app/apikey"),
                                    AIProvider::OpenRouter => (&mut self.openrouter_api_key, "sk-or-...", "https://openrouter.ai/keys"),
                                    AIProvider::LocalLLM => unreachable!(),
                                };

                                ui.label(egui::RichText::new("API Key:").strong());
                                ui.add_space(8.0);

                                ui.add(egui::TextEdit::singleline(api_key)
                                    .password(true)
                                    .hint_text(hint)
                                    .desired_width(400.0));

                                ui.add_space(15.0);

                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Don't have an API key?").color(egui::Color32::GRAY).size(12.0));
                                    if ui.link("Get one here →").clicked() {
                                        let _ = opener::open(get_key_url);
                                    }
                                });
                            }
                        });

                    ui.add_space(30.0);

                    // Check whether we have enough info to proceed
                    let has_key = if is_local_llm {
                        !self.ollama_url.is_empty()
                    } else {
                        let current_key = match self.ai_provider {
                            AIProvider::Groq => &self.groq_api_key,
                            AIProvider::ChatGPT => &self.openai_api_key,
                            AIProvider::Claude => &self.claude_api_key,
                            AIProvider::Gemini => &self.gemini_api_key,
                            AIProvider::OpenRouter => &self.openrouter_api_key,
                            AIProvider::LocalLLM => unreachable!(),
                        };
                        !current_key.trim().is_empty()
                    };

                    let next_btn_label = if is_local_llm { "Test Connection →" } else { "Test API Key →" };

                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(egui::RichText::new("← Back").size(13.0))
                            .min_size(egui::vec2(80.0, 35.0)))
                            .clicked() {
                            self.onboarding_step = OnboardingStep::SelectProvider;
                        }
                        ui.add_space(20.0);
                        if ui.add_enabled(has_key, egui::Button::new(egui::RichText::new(next_btn_label).size(14.0))
                            .fill(if has_key { egui::Color32::from_rgb(59, 130, 246) } else { egui::Color32::from_rgb(71, 85, 105) })
                            .min_size(egui::vec2(160.0, 35.0)))
                            .clicked() {
                            self.onboarding_step = OnboardingStep::TestApiKey;
                            self.validate_current_api_key();
                        }
                    });
                }

                OnboardingStep::TestApiKey => {
                    // Testing API key
                    ui.label(egui::RichText::new("↻").size(48.0).color(egui::Color32::from_rgb(59, 130, 246)));
                    ui.add_space(15.0);
                    ui.label(egui::RichText::new("Testing Your API Key")
                        .strong()
                        .size(24.0));

                    ui.add_space(30.0);

                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(30, 41, 59))
                        .rounding(egui::Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(20.0))
                        .show(ui, |ui| {
                            ui.set_width(400.0);

                            if self.validating_api_key {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.add_space(10.0);
                                    ui.label("Testing connection...");
                                });
                            } else if let Some(result) = &self.api_key_validation_result {
                                if result.starts_with("✓") {
                                    ui.label(egui::RichText::new(result)
                                        .color(egui::Color32::from_rgb(134, 239, 172)));
                                } else {
                                    ui.label(egui::RichText::new(result)
                                        .color(egui::Color32::from_rgb(252, 165, 165)));
                                }
                            }
                        });

                    ui.add_space(30.0);

                    let test_passed = self.api_key_validation_result.as_ref()
                        .map(|r| r.starts_with("✓"))
                        .unwrap_or(false);

                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(egui::RichText::new("← Back").size(13.0))
                            .min_size(egui::vec2(80.0, 35.0)))
                            .clicked() {
                            self.onboarding_step = OnboardingStep::EnterApiKey;
                            self.api_key_validation_result = None;
                        }
                        ui.add_space(20.0);

                        if test_passed {
                            if ui.add(egui::Button::new(egui::RichText::new("Complete Setup →").size(14.0))
                                .fill(egui::Color32::from_rgb(34, 197, 94))
                                .min_size(egui::vec2(160.0, 35.0)))
                                .clicked() {
                                self.onboarding_step = OnboardingStep::Complete;
                            }
                        } else if !self.validating_api_key {
                            if ui.add(egui::Button::new(egui::RichText::new("↻ Retry Test").size(13.0))
                                .min_size(egui::vec2(100.0, 35.0)))
                                .clicked() {
                                self.validate_current_api_key();
                            }
                        }
                    });
                }

                OnboardingStep::Complete => {
                    // Setup complete
                    ui.label(egui::RichText::new("✓").size(64.0).color(egui::Color32::from_rgb(34, 197, 94)));
                    ui.add_space(20.0);
                    ui.label(egui::RichText::new("You're All Set!")
                        .strong()
                        .size(28.0));
                    ui.add_space(15.0);
                    ui.label(egui::RichText::new("Your API key is configured and working.")
                        .size(16.0)
                        .color(egui::Color32::from_rgb(134, 239, 172)));

                    ui.add_space(30.0);

                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(30, 41, 59))
                        .rounding(egui::Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(20.0))
                        .show(ui, |ui| {
                            ui.set_width(400.0);
                            ui.label(egui::RichText::new("Quick Start Guide:").strong());
                            ui.add_space(10.0);
                            ui.label("1. Click 'Add Files' to select receipt images or PDFs");
                            ui.label("2. Customize extraction fields if needed");
                            ui.label("3. Click 'Extract Data' to process your receipts");
                            ui.label("4. Save the results as CSV, Excel, or JSON");
                        });

                    ui.add_space(40.0);

                    if ui.add(egui::Button::new(egui::RichText::new("Start Using the App →").size(16.0))
                        .fill(egui::Color32::from_rgb(34, 197, 94))
                        .min_size(egui::vec2(220.0, 45.0)))
                        .clicked() {
                        self.onboarding_completed = true;
                        self.state = AppState::Idle;
                        self.save_api_keys();
                    }
                }
            }
        });
    }

    fn show_settings_page(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.add_space(20.0);

        // Header with back button
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new(egui::RichText::new("← Back").size(13.0))
                .min_size(egui::vec2(70.0, 28.0)))
                .clicked() {
                self.state = AppState::Idle;
            }
            ui.add_space(20.0);
            ui.heading("Settings");
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);

        // AI Provider Selection
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(30, 41, 59))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(15.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("⚙️ AI Model Selection").strong().size(16.0));
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Select AI Provider:");
                    ui.add_space(10.0);

                    let previous_provider = self.ai_provider;
                    egui::ComboBox::from_id_salt("ai_provider_dropdown")
                        .selected_text(self.ai_provider.name())
                        .width(300.0)
                        .show_ui(ui, |ui| {
                            for provider in AIProvider::all() {
                                ui.selectable_value(&mut self.ai_provider, provider, provider.name());
                            }
                        });

                    // Save settings when provider is changed
                    if self.ai_provider != previous_provider {
                        self.save_api_keys();
                    }
                });

                ui.add_space(10.0);

                // Show current provider description
                ui.label(egui::RichText::new(format!("ℹ️ {}", self.ai_provider.description()))
                    .color(egui::Color32::GRAY)
                    .size(12.0));

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);

                // Model Version Toggle
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.use_latest_models, "Use Latest Models")
                        .on_hover_text(
                            "STABLE (Recommended):\n\
                            • Groq: llama-4-scout (vision-capable)\n\
                            • OpenAI: gpt-4o-mini (cost-effective)\n\
                            • Claude: claude-3-5-sonnet-20241022\n\
                            • Gemini: gemini-2.5-flash\n\
                            • OpenRouter: llama-3.2-90b-vision\n\n\
                            LATEST (Cutting-edge):\n\
                            • Groq: llama-3.3-70b (faster, newer)\n\
                            • OpenAI: gpt-4o (better quality, more $$$)\n\
                            • Claude: claude-3-5-sonnet-latest\n\
                            • Gemini: gemini-flash-latest\n\
                            • OpenRouter: gpt-4o (premium)\n\n\
                            Note: Latest models may change behavior without notice and cost more."
                        )
                        .changed() {
                        self.save_api_keys();
                    }

                    if self.use_latest_models {
                        ui.label(egui::RichText::new("» Latest")
                            .color(egui::Color32::from_rgb(251, 191, 36))
                            .size(11.0));
                    } else {
                        ui.label(egui::RichText::new("● Stable")
                            .color(egui::Color32::from_rgb(34, 197, 94))
                            .size(11.0));
                    }
                });

                ui.add_space(5.0);
                ui.label(egui::RichText::new(
                    if self.use_latest_models {
                        "Using latest models - better quality but may cost more and change without notice"
                    } else {
                        "Using stable models - consistent, predictable, and cost-effective"
                    }
                ).color(egui::Color32::GRAY).size(11.0));

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);

                // Preferences
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.show_tips_on_startup, "Show tips on startup")
                        .on_hover_text("Display helpful tips when the app starts")
                        .changed() {
                        self.save_api_keys();
                        // If enabling, show the tip widget immediately
                        if self.show_tips_on_startup {
                            self.show_tip_widget = true;
                        }
                    }
                });
            });

        ui.add_space(15.0);

        // Debug Logging Section
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(30, 41, 59))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(15.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🐛 Debug Logging").strong().size(16.0));
                });

                ui.add_space(10.0);
                ui.label(egui::RichText::new("Enable detailed logging for troubleshooting").color(egui::Color32::GRAY).size(12.0));

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.enable_debug_logging, "Enable debug logging")
                        .on_hover_text("Write detailed debug information to a log file.\nUseful for troubleshooting issues.")
                        .changed() {
                        crate::debug_log::set_debug_enabled(self.enable_debug_logging);
                        self.save_api_keys();
                    }
                });

                if self.enable_debug_logging {
                    ui.add_space(10.0);

                    // Show log file location and size
                    if let Some(log_path) = crate::debug_log::get_log_file_path() {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Log file:").color(egui::Color32::GRAY));
                            ui.label(egui::RichText::new(log_path.to_string_lossy()).color(egui::Color32::LIGHT_GRAY).size(11.0));
                        });

                        if let Some(size) = crate::debug_log::get_log_size() {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Size:").color(egui::Color32::GRAY));
                                ui.label(egui::RichText::new(crate::debug_log::format_file_size(size)).color(egui::Color32::LIGHT_GRAY));
                            });
                        }
                    }

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        // Open log file in default text editor
                        if ui.add(egui::Button::new(egui::RichText::new("Open Log File").size(12.0))
                            .min_size(egui::vec2(95.0, 26.0)))
                            .clicked() {
                            if let Some(log_path) = crate::debug_log::get_log_file_path() {
                                let _ = opener::open(&log_path);
                            }
                        }
                        ui.add_space(8.0);

                        // Clear log file
                        if ui.add(egui::Button::new(egui::RichText::new("Clear Log").size(12.0))
                            .min_size(egui::vec2(80.0, 26.0)))
                            .clicked() {
                            let _ = crate::debug_log::clear_log();
                        }
                    });
                }
            });

        ui.add_space(15.0);

        // Export Format Section
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(30, 41, 59))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(15.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📄 Export Format").strong().size(16.0));
                });

                ui.add_space(10.0);
                ui.label(egui::RichText::new("Choose the format for exported data files").color(egui::Color32::GRAY).size(12.0));

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label("Format:");
                    egui::ComboBox::from_id_salt("export_format_selector")
                        .selected_text(self.export_format.name())
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(&mut self.export_format, ExportFormat::Csv, "CSV")
                                .on_hover_text("Comma-separated values - opens in Excel, Google Sheets, etc.")
                                .changed() {
                                self.save_api_keys();
                            }
                            if ui.selectable_value(&mut self.export_format, ExportFormat::Excel, "Excel")
                                .on_hover_text("Native Excel format (.xlsx) with separate sheets for summary and items")
                                .changed() {
                                self.save_api_keys();
                            }
                            if ui.selectable_value(&mut self.export_format, ExportFormat::Json, "JSON")
                                .on_hover_text("JSON format - ideal for developers and data processing")
                                .changed() {
                                self.save_api_keys();
                            }
                        });
                });

                ui.add_space(8.0);

                // Format description
                let (format_hint, format_detail) = match self.export_format {
                    ExportFormat::Csv => (
                        "Creates 2 separate files:",
                        "• _summary.csv (one row per receipt)\n• _items.csv (one row per item)"
                    ),
                    ExportFormat::Excel => (
                        "Creates 1 file with 2 sheets:",
                        "• \"Summary\" sheet (one row per receipt)\n• \"Items\" sheet (one row per item)"
                    ),
                    ExportFormat::Json => (
                        "Creates 2 separate files:",
                        "• _summary.json (array of receipt objects)\n• _items.json (array of item objects)"
                    ),
                };
                ui.label(egui::RichText::new(format_hint).color(egui::Color32::LIGHT_GRAY).size(11.0));
                ui.label(egui::RichText::new(format_detail).color(egui::Color32::GRAY).size(10.0));
            });

        ui.add_space(15.0);

        // API Keys Section - Collapsible
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(30, 41, 59))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(15.0))
            .show(ui, |ui| {
                egui::CollapsingHeader::new(
                    egui::RichText::new("🔑 API Keys").strong().size(16.0)
                )
                .default_open(self.show_api_keys_section)
                .show(ui, |ui| {
                    ui.add_space(5.0);
                    ui.label(egui::RichText::new("Enter your API keys for the AI providers you want to use").color(egui::Color32::GRAY).size(12.0));

                    // Clickable link to help
                    ui.horizontal(|ui| {
                        if ui.link("How to get an API key?").clicked() {
                            self.show_help_modal = true;
                        }
                    });

                    ui.add_space(15.0);

                // Groq API Key
                ui.label("Groq API Key:");
                ui.add_space(4.0);
                if ui.add(egui::TextEdit::singleline(&mut self.groq_api_key)
                    .hint_text("gsk_...")
                    .password(true)
                    .desired_width(ui.available_width())).changed() {
                    self.save_api_keys();
                }

                ui.add_space(10.0);

                // OpenAI API Key
                ui.label("OpenAI API Key:");
                ui.add_space(4.0);
                if ui.add(egui::TextEdit::singleline(&mut self.openai_api_key)
                    .hint_text("sk-...")
                    .password(true)
                    .desired_width(ui.available_width())).changed() {
                    self.save_api_keys();
                }

                ui.add_space(10.0);

                // Claude API Key
                ui.label("Anthropic Claude API Key:");
                ui.add_space(4.0);
                if ui.add(egui::TextEdit::singleline(&mut self.claude_api_key)
                    .hint_text("sk-ant-...")
                    .password(true)
                    .desired_width(ui.available_width())).changed() {
                    self.save_api_keys();
                }

                ui.add_space(10.0);

                // Gemini API Key
                ui.label("Google Gemini API Key:");
                ui.add_space(4.0);
                if ui.add(egui::TextEdit::singleline(&mut self.gemini_api_key)
                    .hint_text("AIza...")
                    .password(true)
                    .desired_width(ui.available_width())).changed() {
                    self.save_api_keys();
                }

                ui.add_space(10.0);

                // OpenRouter API Key
                ui.label("OpenRouter API Key:");
                ui.add_space(4.0);
                if ui.add(egui::TextEdit::singleline(&mut self.openrouter_api_key)
                    .hint_text("sk-or-...")
                    .password(true)
                    .desired_width(ui.available_width())).changed() {
                    self.save_api_keys();
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // Local LLM (Ollama / LM Studio / Jan)
                ui.label(egui::RichText::new("🖥 Local LLM (Ollama / LM Studio / Jan)").strong());
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Server URL:").size(12.0));
                ui.add_space(4.0);
                if ui.add(egui::TextEdit::singleline(&mut self.ollama_url)
                    .hint_text("http://localhost:11434")
                    .desired_width(ui.available_width())).changed() {
                    self.save_api_keys();
                }
                ui.add_space(6.0);
                ui.label(egui::RichText::new("Model Name:").size(12.0));
                ui.add_space(4.0);
                if ui.add(egui::TextEdit::singleline(&mut self.local_model_name)
                    .hint_text("llava")
                    .desired_width(ui.available_width())).changed() {
                    self.save_api_keys();
                }
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Use a vision-capable model such as llava or llama3.2-vision")
                    .color(egui::Color32::GRAY).size(11.0));

                ui.add_space(15.0);

                // Validate API Key button
                ui.horizontal(|ui| {
                    let current_key = match self.ai_provider {
                        AIProvider::Groq => &self.groq_api_key,
                        AIProvider::ChatGPT => &self.openai_api_key,
                        AIProvider::Claude => &self.claude_api_key,
                        AIProvider::Gemini => &self.gemini_api_key,
                        AIProvider::OpenRouter => &self.openrouter_api_key,
                        AIProvider::LocalLLM => &self.ollama_url,
                    };

                    let can_validate = !current_key.is_empty() && !self.validating_api_key;
                    let btn_label = if self.ai_provider == AIProvider::LocalLLM { "Test Connection" } else { "Test API Key" };

                    if ui.add_enabled(can_validate, egui::Button::new(egui::RichText::new(btn_label).size(12.0))
                        .min_size(egui::vec2(105.0, 26.0)))
                        .clicked() {
                        self.validate_current_api_key();
                    }

                    if self.validating_api_key {
                        ui.spinner();
                        ui.label("Testing...");
                    }
                });

                ui.add_space(8.0);

                // Show validation result
                if let Some(result) = &self.api_key_validation_result {
                    if result.starts_with("✓") {
                        ui.label(egui::RichText::new(result).color(egui::Color32::from_rgb(34, 197, 94)));
                    } else {
                        ui.label(egui::RichText::new(result).color(egui::Color32::from_rgb(239, 68, 68)));
                    }
                }
                });

                // Track the collapsed/expanded state
                let is_open = ui.memory(|mem| {
                    mem.data.get_temp::<bool>(egui::Id::new("api_keys_section_open"))
                        .unwrap_or(self.show_api_keys_section)
                });

                if is_open != self.show_api_keys_section {
                    self.show_api_keys_section = is_open;
                    self.save_api_keys();
                }
            });

        ui.add_space(15.0);

        // Getting Started / Help Section
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(30, 41, 59))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(15.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new(":: Getting Started Guide").strong().size(16.0));
                ui.add_space(10.0);

                // How to Use the App
                egui::CollapsingHeader::new(egui::RichText::new("→ How to Use This App").size(14.0))
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("Step 1: Set Up Your API Key").strong());
                        ui.label("  1. Choose an AI provider from the dropdown above (Groq, OpenAI, Claude, Gemini, or OpenRouter)");
                        ui.label("  2. Get an API key from your chosen provider (see 'How to Get API Keys' below)");
                        ui.label("  3. Paste your API key in the corresponding field above");
                        ui.label("  4. Click '🔍 Test API Key' to verify it works");

                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Step 2: Configure Extraction Fields").strong());
                        ui.label("  1. Go back to the main screen (click '← Back')");
                        ui.label("  2. In the 'Extraction Fields' section, add the data fields you want to extract");
                        ui.label("  3. Examples: Date, Merchant, Total, Tax, Items, Payment Method, etc.");
                        ui.label("  4. Click '+ Add Field' to add each field");

                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Step 3: Select Your Files").strong());
                        ui.label("  1. Click 'Add Files' to choose receipt images or PDFs");
                        ui.label("  2. You can select multiple files at once");
                        ui.label("  3. Supported formats: JPG, PNG, PDF");

                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Step 4: Set Output Location").strong());
                        ui.label("  1. In the 'CSV Output' section, click 'Browse'");
                        ui.label("  2. Select where you want to save the extracted data");
                        ui.label("  3. The app will auto-save after each extraction to prevent data loss");

                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Step 5: Extract Data").strong());
                        ui.label("  1. Click 'Extract Data' to start processing");
                        ui.label("  2. The app will process each file and extract the requested fields");
                        ui.label("  3. Watch the progress bar to see how many files are complete");
                        ui.label("  4. If rate limits are hit, the app will automatically wait and retry");

                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Step 6: Review and Save").strong());
                        ui.label("  1. After processing, review any incomplete extractions");
                        ui.label("  2. Fill in missing data manually if needed");
                        ui.label("  3. Click 'Save CSV' to save your results");
                        ui.add_space(5.0);
                    });

                ui.add_space(8.0);

                // How to Get API Keys
                egui::CollapsingHeader::new(egui::RichText::new("🔑 How to Get API Keys").size(14.0))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add_space(5.0);

                        // Groq
                        ui.label(egui::RichText::new("Groq (Recommended for Free Tier)").strong());
                        ui.label("  • Website: https://console.groq.com/keys");
                        ui.label("  • Free tier: 30 requests/minute, very fast");
                        ui.label("  • Steps:");
                        ui.label("    1. Sign up at https://groq.com");
                        ui.label("    2. Go to https://console.groq.com/keys");
                        ui.label("    3. Click 'Create API Key'");
                        ui.label("    4. Copy the key (starts with 'gsk_')");

                        ui.add_space(8.0);

                        // OpenAI
                        ui.label(egui::RichText::new("OpenAI (ChatGPT)").strong());
                        ui.label("  • Website: https://platform.openai.com/api-keys");
                        ui.label("  • Paid service: ~$0.15 per 1000 images");
                        ui.label("  • Steps:");
                        ui.label("    1. Sign up at https://platform.openai.com");
                        ui.label("    2. Add payment method");
                        ui.label("    3. Go to https://platform.openai.com/api-keys");
                        ui.label("    4. Click '+ Create new secret key'");
                        ui.label("    5. Copy the key (starts with 'sk-')");

                        ui.add_space(8.0);

                        // Claude
                        ui.label(egui::RichText::new("Anthropic Claude").strong());
                        ui.label("  • Website: https://console.anthropic.com/settings/keys");
                        ui.label("  • Paid service: ~$0.80 per 1000 images");
                        ui.label("  • Steps:");
                        ui.label("    1. Sign up at https://console.anthropic.com");
                        ui.label("    2. Add payment method");
                        ui.label("    3. Go to https://console.anthropic.com/settings/keys");
                        ui.label("    4. Click 'Create Key'");
                        ui.label("    5. Copy the key (starts with 'sk-ant-')");

                        ui.add_space(8.0);

                        // Gemini
                        ui.label(egui::RichText::new("Google Gemini").strong());
                        ui.label("  • Website: https://aistudio.google.com/app/apikey");
                        ui.label("  • Free tier: 15 requests/minute, 20 requests/day");
                        ui.label("  • Steps:");
                        ui.label("    1. Sign in with Google account");
                        ui.label("    2. Go to https://aistudio.google.com/app/apikey");
                        ui.label("    3. Click 'Create API Key'");
                        ui.label("    4. Copy the key (starts with 'AIza')");
                        ui.label("  • Note: Free tier has daily limits!");

                        ui.add_space(8.0);

                        // OpenRouter
                        ui.label(egui::RichText::new("OpenRouter").strong());
                        ui.label("  • Website: https://openrouter.ai/keys");
                        ui.label("  • Pay-per-use: Access to multiple models");
                        ui.label("  • Steps:");
                        ui.label("    1. Sign up at https://openrouter.ai");
                        ui.label("    2. Add credits to your account");
                        ui.label("    3. Go to https://openrouter.ai/keys");
                        ui.label("    4. Click 'Create Key'");
                        ui.label("    5. Copy the key (starts with 'sk-or-')");

                        ui.add_space(5.0);
                    });

                ui.add_space(8.0);

                // Tips and Best Practices
                egui::CollapsingHeader::new(egui::RichText::new("★ Tips & Best Practices").size(14.0))
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.add_space(5.0);
                        ui.label("• Start with Groq or Gemini for free testing");
                        ui.label("• Enable 'Auto-save' to prevent data loss during processing");
                        ui.label("• Use 'Stable' models for consistent results");
                        ui.label("• Use 'Latest' models for cutting-edge performance");
                        ui.label("• If you hit rate limits, the app will automatically wait and retry");
                        ui.label("• For best results, use clear, well-lit receipt images");
                        ui.label("• You can process multiple files at once");
                        ui.label("• Review incomplete extractions and fill in missing data manually");
                        ui.add_space(5.0);
                    });

                ui.add_space(8.0);

                // Important Note
                ui.separator();
                ui.add_space(8.0);
                ui.label(egui::RichText::new("! Important").strong().color(egui::Color32::from_rgb(251, 191, 36)));
                ui.label(egui::RichText::new("You must provide your own API keys to use this app. API usage costs are your responsibility.").color(egui::Color32::GRAY).size(11.0));
                ui.label(egui::RichText::new("Free tiers are available for Groq and Gemini, but have rate limits.").color(egui::Color32::GRAY).size(11.0));
            });
    }

    fn validate_current_api_key(&mut self) {
        // Handle Local LLM separately - test the server connection
        if self.ai_provider == AIProvider::LocalLLM {
            if self.ollama_url.is_empty() {
                self.api_key_validation_result = Some("✗ Please enter the Local LLM server URL".to_string());
                return;
            }
            let url = self.ollama_url.clone();
            let model = if self.local_model_name.is_empty() { "llava".to_string() } else { self.local_model_name.clone() };
            self.validating_api_key = true;
            self.api_key_validation_result = Some("... Testing Local LLM connection...".to_string());
            let (tx, rx) = channel();
            self.api_validation_rx = Some(rx);
            let use_latest = self.use_latest_models;
            self.runtime.spawn(async move {
                use crate::ai_extractor::AIExtractor;
                let extractor = AIExtractor::new_with_options(String::new(), use_latest)
                    .with_local_llm(url, model);
                let result = match extractor.test_api_key().await {
                    Ok(resp) => Ok(format!("✓ Local LLM connected! Response: {}", resp.chars().take(80).collect::<String>())),
                    Err(e) => Err(format!("✗ Connection failed: {}", e)),
                };
                let _ = tx.send(result);
            });
            return;
        }

        let (api_key, provider_name) = match self.ai_provider {
            AIProvider::Groq => (self.groq_api_key.clone(), "Groq"),
            AIProvider::ChatGPT => (self.openai_api_key.clone(), "OpenAI"),
            AIProvider::Claude => (self.claude_api_key.clone(), "Claude"),
            AIProvider::Gemini => (self.gemini_api_key.clone(), "Gemini"),
            AIProvider::OpenRouter => (self.openrouter_api_key.clone(), "OpenRouter"),
            AIProvider::LocalLLM => unreachable!(),
        };

        if api_key.is_empty() {
            self.api_key_validation_result = Some(format!("✗ Please enter a {} API key first", provider_name));
            return;
        }

        // Check API key format first
        let is_valid_format = match self.ai_provider {
            AIProvider::Groq => api_key.starts_with("gsk_"),
            AIProvider::ChatGPT => api_key.starts_with("sk-"),
            AIProvider::Claude => api_key.starts_with("sk-ant-"),
            AIProvider::Gemini => api_key.starts_with("AIza"),
            AIProvider::OpenRouter => api_key.starts_with("sk-or-"),
            AIProvider::LocalLLM => unreachable!(),
        };

        if !is_valid_format {
            self.api_key_validation_result = Some(format!("✗ {} API key format is invalid. Expected format: {}",
                provider_name,
                match self.ai_provider {
                    AIProvider::Groq => "gsk_...",
                    AIProvider::ChatGPT => "sk-...",
                    AIProvider::Claude => "sk-ant-...",
                    AIProvider::Gemini => "AIza...",
                    AIProvider::OpenRouter => "sk-or-...",
                    AIProvider::LocalLLM => unreachable!(),
                }
            ));
            return;
        }

        // Format is valid, now test the API with a real request
        self.validating_api_key = true;
        self.api_key_validation_result = Some(format!("... Testing {} API key...", provider_name));

        // Create channel for validation result
        let (tx, rx) = channel();
        self.api_validation_rx = Some(rx);

        // Capture use_latest_models flag
        let use_latest_models = self.use_latest_models;

        // Spawn async task to test the API key
        self.runtime.spawn(async move {
            use crate::ai_extractor::AIExtractor;

            let extractor = AIExtractor::new_with_options(api_key, use_latest_models);
            let result = match extractor.test_api_key().await {
                Ok(response) => {
                    Ok(format!("✓ {} API key is working! Response: {}", provider_name,
                        response.chars().take(100).collect::<String>()))
                }
                Err(e) => {
                    Err(format!("✗ {} API key test failed: {}", provider_name, e))
                }
            };
            let _ = tx.send(result);
        });
    }

    fn start_extraction(&mut self) {
        if self.selected_files.is_empty() {
            self.error_message = "Please select at least one document.".to_string();
            self.state = AppState::Error;
            return;
        }

        // Pre-check: Test API availability before starting
        println!("🔍 Checking API availability before processing...");
        let api_key = match self.ai_provider {
            AIProvider::Groq => self.groq_api_key.clone(),
            AIProvider::ChatGPT => self.openai_api_key.clone(),
            AIProvider::Claude => self.claude_api_key.clone(),
            AIProvider::Gemini => self.gemini_api_key.clone(),
            AIProvider::OpenRouter => self.openrouter_api_key.clone(),
            AIProvider::LocalLLM => String::new(),
        };

        let use_latest_models = self.use_latest_models;
        let is_local_llm = self.ai_provider == AIProvider::LocalLLM;
        let ollama_url_precheck = self.ollama_url.clone();
        let local_model_precheck = if self.local_model_name.is_empty() { "llava".to_string() } else { self.local_model_name.clone() };

        let mut ai = AIExtractor::new_with_options(api_key.clone(), use_latest_models);
        if is_local_llm {
            ai = ai.with_local_llm(ollama_url_precheck, local_model_precheck);
        }

        // Quick synchronous check using runtime
        let check_result = self.runtime.block_on(async {
            ai.test_api_key().await
        });

        if let Err(e) = check_result {
            let error_msg = e.to_string();
            if error_msg.contains("429") || error_msg.contains("RESOURCE_EXHAUSTED") || error_msg.contains("rate") {
                // Extract wait time if available
                let wait_seconds = if let Some(retry_pos) = error_msg.to_lowercase().find("retry in ") {
                    // Find position in lowercase, then extract from original
                    let after_retry = &error_msg[retry_pos + 9..];
                    if let Some(s_pos) = after_retry.find('s') {
                        let num_str = &after_retry[..s_pos];
                        num_str.parse::<f64>().unwrap_or(60.0).ceil() as u64
                    } else {
                        60
                    }
                } else {
                    60
                };

                self.rate_limit_warning = Some(format!(
                    "! API rate limit detected! You need to wait approximately {} seconds before processing. The app will automatically retry when you start processing.",
                    wait_seconds
                ));
                println!("! Pre-check detected rate limit: wait {} seconds", wait_seconds);
                // Don't return - let user decide to proceed (it will wait automatically)
            } else {
                println!("! API pre-check failed (non-rate-limit): {}", error_msg);
                // For non-rate-limit errors, show warning but allow processing
                self.rate_limit_warning = Some(format!("! API check warning: {}", error_msg));
            }
        } else {
            println!("[OK] API pre-check passed");
            self.rate_limit_warning = None;
        }

        self.state = AppState::Processing;
        self.current_progress = 0;
        self.total_progress = self.selected_files.len();
        self.progress_message = format!("Processing 0/{} documents...", self.total_progress);

        // Create channel for progress updates
        let (tx, rx) = channel();
        self.progress_rx = Some(rx);

        // Spawn async processing
        let files = self.selected_files.clone();
        // Extract field names for the ExtractionOptions (which expects Vec<String>)
        let field_names = get_field_names(&self.extraction_fields);
        let item_field_names = get_field_names(&self.item_fields);
        // Extract custom prompts for AI-enhanced fields
        let mut custom_prompts = get_custom_prompts(&self.extraction_fields);
        custom_prompts.extend(get_custom_prompts(&self.item_fields));
        let enable_general = self.extraction_mode == ExtractionMode::General;
        let enable_detailed = self.extraction_mode == ExtractionMode::Detailed;

        // Determine API key based on provider
        let api_key = match self.ai_provider {
            AIProvider::Groq => self.groq_api_key.clone(),
            AIProvider::ChatGPT => self.openai_api_key.clone(),
            AIProvider::Claude => self.claude_api_key.clone(),
            AIProvider::Gemini => self.gemini_api_key.clone(),
            AIProvider::OpenRouter => self.openrouter_api_key.clone(),
            AIProvider::LocalLLM => String::new(),
        };

        let ollama_url_thread = self.ollama_url.clone();
        let local_model_thread = if self.local_model_name.is_empty() { "llava".to_string() } else { self.local_model_name.clone() };

        let processor = self.processor.clone();
        let total_files = files.len();
        let use_latest_models = self.use_latest_models;
        let export_format = self.export_format;
        let csv_output_path = if !self.csv_output_path.is_empty() {
            Some(PathBuf::from(&self.csv_output_path))
        } else {
            None
        };

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut proc = DocumentProcessor::new();
                let mut ai = AIExtractor::new_with_options(api_key, use_latest_models);
                if is_local_llm {
                    ai = ai.with_local_llm(ollama_url_thread, local_model_thread);
                }

                // Create extraction options for dual extraction
                let options = ExtractionOptions {
                    enable_general,
                    enable_detailed,
                    summary_fields: field_names.clone(),
                    item_fields: item_field_names.clone(),
                    custom_prompts,
                };

                for (idx, file) in files.iter().enumerate() {
                    let file_name = file.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    println!("Processing {}/{}: {:?}", idx + 1, total_files, file);
                    let _ = tx.send(ProcessingMessage::Progress(idx + 1, total_files, file_name));

                    // Use the new dual extraction method
                    if let Err(e) = proc.process_document_with_options(file, &options, &mut ai).await {
                        eprintln!("Error processing {:?}: {}", file, e);
                        let error_msg = e.to_string();

                        // If rate limited, extract wait time from error message and wait
                        if error_msg.contains("rate") || error_msg.contains("429") || error_msg.contains("Rate") || error_msg.contains("RESOURCE_EXHAUSTED") {
                            // Try to extract retry delay from error message
                            let wait_seconds = if let Some(retry_pos) = error_msg.to_lowercase().find("retry in ") {
                                // Find position in lowercase, then extract from original
                                let after_retry = &error_msg[retry_pos + 9..];
                                if let Some(s_pos) = after_retry.find('s') {
                                    let num_str = &after_retry[..s_pos];
                                    num_str.parse::<f64>().unwrap_or(60.0).ceil() as u64
                                } else {
                                    60
                                }
                            } else if error_msg.contains("retryDelay") {
                                // Gemini format: "retryDelay": "59s"
                                60
                            } else {
                                60
                            };

                            println!("⏳ Rate limit hit! Waiting {} seconds before retry...", wait_seconds);

                            // Send countdown updates every second
                            for remaining in (1..=wait_seconds).rev() {
                                let _ = tx.send(ProcessingMessage::RateLimitCountdown(remaining));
                                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            }

                            // Retry once
                            println!("[RETRY] Retrying {:?}...", file.file_name().unwrap_or_default());
                            if let Err(e2) = proc.process_document_with_options(file, &options, &mut ai).await {
                                let _ = tx.send(ProcessingMessage::Error(format!("[ERROR] Error processing {:?}: {}", file.file_name().unwrap_or_default(), e2)));
                            } else {
                                println!("[OK] Successfully processed {:?} after retry", file.file_name().unwrap_or_default());
                            }
                        } else {
                            let _ = tx.send(ProcessingMessage::Error(format!("[ERROR] Error processing {:?}: {}", file.file_name().unwrap_or_default(), e)));
                        }
                    } else {
                        println!("[OK] Successfully processed {:?}", file.file_name().unwrap_or_default());

                        // Auto-save after each successful extraction when path is set
                        if csv_output_path.is_some() {
                            let save_path = csv_output_path.as_ref().unwrap();
                            // Use selected format for auto-save
                            match CSVExporter::export_with_format(
                                proc.get_results(),
                                &field_names,
                                &item_field_names,
                                enable_general,
                                enable_detailed,
                                save_path.clone(),
                                export_format,
                            ) {
                                Ok(result) => {
                                    if let Some(summary_path) = &result.summary_path {
                                        println!("[SAVE] Auto-saved summary to: {:?}", summary_path);
                                    }
                                    if let Some(items_path) = &result.items_path {
                                        println!("[SAVE] Auto-saved items to: {:?}", items_path);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[WARN] Auto-save failed: {}", e);
                                }
                            }
                        }
                    }

                    // Delay between receipts to avoid rate limiting
                    // Gemini free tier: 15 RPM = 4 seconds between requests
                    // Groq free tier: 30 RPM = 2 seconds between requests
                    if idx < files.len() - 1 {
                        tokio::time::sleep(std::time::Duration::from_millis(4000)).await;
                    }
                }

                // Store processor results and signal ready to save
                *processor.lock().unwrap() = Some(proc);
                let _ = tx.send(ProcessingMessage::ProcessingComplete);
            });
        });
    }
}

impl eframe::App for ReceiptExtractorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts - collect actions first, then execute
        let (open_files, save_csv, toggle_settings, toggle_help, close_modal) = ctx.input(|i| {
            let ctrl_or_cmd = i.modifiers.ctrl || i.modifiers.mac_cmd;

            let open_files = ctrl_or_cmd && i.key_pressed(egui::Key::O);
            let save_csv = ctrl_or_cmd && i.key_pressed(egui::Key::S);
            let toggle_settings = ctrl_or_cmd && i.key_pressed(egui::Key::Comma);
            let toggle_help = i.key_pressed(egui::Key::F1);
            let close_modal = i.key_pressed(egui::Key::Escape);

            (open_files, save_csv, toggle_settings, toggle_help, close_modal)
        });

        // Execute keyboard shortcut actions
        if open_files && self.state == AppState::Idle {
            self.add_more_files();
        }

        if save_csv {
            // Check if we have results to save
            let has_results = self.processor.lock().unwrap().as_ref()
                .map(|p| !p.get_results().is_empty())
                .unwrap_or(false);
            if has_results {
                self.save_csv();
            }
        }

        if toggle_settings {
            if self.state == AppState::Settings {
                self.state = AppState::Idle;
            } else {
                self.state = AppState::Settings;
            }
        }

        if toggle_help {
            self.show_help_modal = !self.show_help_modal;
        }

        if close_modal {
            if self.show_help_modal {
                self.show_help_modal = false;
            } else if self.show_about_modal {
                self.show_about_modal = false;
            } else if self.state == AppState::Settings {
                self.state = AppState::Idle;
            }
        }

        // Check for field enhancement results from background tasks
        if let Some(rx) = &self.enhancement_rx {
            if let Ok((field_idx, prompt, is_item_field)) = rx.try_recv() {
                let fields = if is_item_field {
                    &mut self.item_fields
                } else {
                    &mut self.extraction_fields
                };

                if let Some(field) = fields.get_mut(field_idx) {
                    field.extraction_prompt = prompt;
                    field.is_enhancing = false;

                    // Save updated fields
                    if is_item_field {
                        save_item_fields(&self.item_fields);
                    } else {
                        save_extraction_fields(&self.extraction_fields);
                    }
                }
            }
        }

        // Handle drag and drop files
        // Note: Drag and drop from OS file manager does NOT work on Linux (Wayland/X11)
        // This is a known limitation in winit. See: https://github.com/rust-windowing/winit/issues/1881
        // The code below will work on Windows/macOS but not on Linux.
        let dropped_files: Vec<PathBuf> = ctx.input(|i| {
            i.raw.dropped_files.iter()
                .filter_map(|f| f.path.clone())
                .filter(|p| {
                    // Only accept supported file types
                    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                        matches!(ext.to_lowercase().as_str(), "pdf" | "png" | "jpg" | "jpeg" | "txt" | "html" | "htm")
                    } else {
                        false
                    }
                })
                .collect()
        });

        if !dropped_files.is_empty() {
            for file in dropped_files {
                if !self.selected_files.contains(&file) {
                    self.selected_files.push(file);
                }
            }
        }

        // Check for API validation updates
        let mut should_clear_api_rx = false;
        if let Some(rx) = &self.api_validation_rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(success_msg) => {
                        self.api_key_validation_result = Some(success_msg);
                        self.validating_api_key = false;
                    }
                    Err(error_msg) => {
                        self.api_key_validation_result = Some(error_msg);
                        self.validating_api_key = false;
                    }
                }
                should_clear_api_rx = true;
            }
        }
        if should_clear_api_rx {
            self.api_validation_rx = None;
        }

        // Check for file dialog results
        let mut should_clear_file_dialog_rx = false;
        if let Some(rx) = &self.file_dialog_rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    FileDialogResult::FilesToAdd(files) => {
                        // Save the directory from the first file for next time
                        if let Some(first_file) = files.first() {
                            if let Some(parent) = first_file.parent() {
                                self.last_open_directory = Some(parent.to_path_buf());
                                self.save_api_keys();
                            }
                        }
                        // Add files that aren't already in the list
                        for file in files {
                            if !self.selected_files.contains(&file) {
                                self.selected_files.push(file);
                            }
                        }
                    }
                    FileDialogResult::CsvPathSelected(path) => {
                        // Save the directory for next time
                        if let Some(parent) = path.parent() {
                            self.last_save_directory = Some(parent.to_path_buf());
                            self.save_api_keys();
                        }
                        self.csv_output_path = path.to_string_lossy().to_string();
                    }
                }
                should_clear_file_dialog_rx = true;
            }
        }
        if should_clear_file_dialog_rx {
            self.file_dialog_rx = None;
        }

        // Check for progress updates
        let mut should_clear_rx = false;
        if let Some(rx) = &self.progress_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    ProcessingMessage::Progress(current, total, file_name) => {
                        self.current_progress = current;
                        self.total_progress = total;
                        self.progress_message = format!("Processing {}/{}: {}...", current, total, file_name);
                    }
                    ProcessingMessage::ProcessingComplete => {
                        // Clear rate limit state
                        self.rate_limit_countdown = None;
                        self.rate_limit_warning = None;

                        // Check for incomplete extractions
                        self.incomplete_extractions.clear();
                        if let Some(proc) = self.processor.lock().unwrap().as_ref() {
                            let critical_fields = ["Vendor/Store Name", "Total Amount", "item"];
                            for result in proc.get_results() {
                                let mut missing: Vec<String> = Vec::new();
                                for field in &critical_fields {
                                    if let Some(value) = result.fields.get(*field) {
                                        if value.trim().is_empty() {
                                            missing.push(field.to_string());
                                        }
                                    } else {
                                        missing.push(field.to_string());
                                    }
                                }
                                // Track incomplete extractions with error reason
                                if !missing.is_empty() || result.extraction_error.is_some() {
                                    self.incomplete_extractions.push(IncompleteExtraction {
                                        file_name: result.file_name.clone(),
                                        missing_fields: missing,
                                        error_reason: result.extraction_error.clone(),
                                    });
                                }
                            }
                        }
                        self.state = AppState::ReadyToSave;
                        // Trigger save button pulse animation
                        self.save_button_pulse_start = Some(std::time::Instant::now());
                        should_clear_rx = true;
                    }
                    ProcessingMessage::Error(err) => {
                        self.error_message = err;
                        self.state = AppState::Error;
                        should_clear_rx = true;
                    }
                    ProcessingMessage::RateLimitCountdown(seconds) => {
                        self.rate_limit_countdown = Some(seconds);
                        self.progress_message = format!("⏳ Rate limited. Retrying in {} seconds...", seconds);
                    }
                }
            }
        }
        if should_clear_rx {
            self.progress_rx = None;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(15, 23, 42))
                .inner_margin(egui::Margin::symmetric(20.0, 10.0)))  // Add horizontal and vertical padding
            .show(ctx, |ui| {
            // Wrap everything in a vertical scroll area for the main content
            egui::ScrollArea::vertical()
                .id_salt("main_scroll_area")
                .auto_shrink([false, false])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    // Add right margin to prevent content from overlapping scrollbar
                    ui.set_width(ui.available_width() - 12.0);
            // Onboarding Wizard
            if self.state == AppState::Onboarding {
                self.show_onboarding_wizard(ctx, ui);
                return;
            }

            // Settings Page
            if self.state == AppState::Settings {
                self.show_settings_page(ctx, ui);
                return;
            }

            // Main App UI - Header
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.heading("🧾 Receipt Data Extractor");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Settings button
                    if ui.add(egui::Button::new(egui::RichText::new("⚙️ Settings").size(13.0))
                        .min_size(egui::vec2(85.0, 28.0)))
                        .on_hover_text("Open settings (Ctrl+,)")
                        .clicked() {
                        self.state = AppState::Settings;
                    }
                    ui.add_space(4.0);

                    // Help button
                    if ui.add(egui::Button::new(egui::RichText::new("? Help").size(13.0))
                        .min_size(egui::vec2(70.0, 28.0)))
                        .on_hover_text("View setup instructions and API key guides (F1)")
                        .clicked() {
                        self.show_help_modal = true;
                    }
                    ui.add_space(4.0);

                    // About button
                    if ui.add(egui::Button::new(egui::RichText::new("ℹ️ About").size(13.0))
                        .min_size(egui::vec2(70.0, 28.0)))
                        .on_hover_text("About Receipt Data Extractor")
                        .clicked() {
                        self.show_about_modal = true;
                    }
                });
            });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(15.0);

            // CSV Output Path Section (always visible)
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(30, 41, 59))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("💾 CSV Output").strong());
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        // Text input for path
                        let text_edit = egui::TextEdit::singleline(&mut self.csv_output_path)
                            .hint_text("Set output path or leave empty to choose on save...")
                            .desired_width(ui.available_width() - 80.0);
                        ui.add(text_edit);

                        // Browse button
                        if ui.add(egui::Button::new(egui::RichText::new("Browse").size(12.0))
                            .min_size(egui::vec2(65.0, 24.0)))
                            .clicked() {
                            self.browse_csv_path();
                        }
                    });

                    ui.add_space(8.0);

                    // Export format selector
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Export Format:").size(12.0));
                        ui.add_space(8.0);
                        let format_changed = egui::ComboBox::from_id_salt("export_format_selector")
                            .selected_text(self.export_format.name())
                            .show_ui(ui, |ui| {
                                let mut changed = false;
                                for format in [ExportFormat::Csv, ExportFormat::Excel, ExportFormat::Json] {
                                    if ui.selectable_value(&mut self.export_format, format, format.name()).changed() {
                                        changed = true;
                                    }
                                }
                                changed
                            }).inner.unwrap_or(false);
                        if format_changed {
                            self.save_api_keys();
                        }
                    });

                    ui.add_space(10.0);

                    // Extract Data Button
                    let has_required_fields = if self.extraction_mode == ExtractionMode::General {
                        !self.extraction_fields.is_empty()
                    } else {
                        !self.item_fields.is_empty()
                    };
                    let can_extract = !self.selected_files.is_empty() && has_required_fields && self.state != AppState::Processing;
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                            let button = egui::Button::new(
                                egui::RichText::new("→ Extract Data")
                                    .size(16.0)
                                    .color(if can_extract { egui::Color32::WHITE } else { egui::Color32::GRAY })
                            )
                            .fill(if can_extract { egui::Color32::from_rgb(34, 197, 94) } else { egui::Color32::from_rgb(71, 85, 105) })
                            .min_size(egui::vec2(280.0, 40.0));

                            if ui.add_enabled(can_extract, button).clicked() {
                                self.start_extraction();
                            }
                        });
                    });

                    // Show save error if any
                    if let Some(error) = &self.csv_save_error.clone() {
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new(error).color(egui::Color32::from_rgb(239, 68, 68)).size(12.0));
                    }

                    // Show save button if we have results to save
                    let has_results = self.processor.lock().unwrap().as_ref()
                        .map(|p| !p.get_results().is_empty())
                        .unwrap_or(false);

                    if has_results {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            // Calculate pulse animation for Save button
                            let button_color = if let Some(pulse_start) = self.save_button_pulse_start {
                                let elapsed = pulse_start.elapsed().as_secs_f32();
                                if elapsed < 2.0 {
                                    // Pulse between bright green and normal green
                                    let pulse = ((elapsed * 4.0).sin() * 0.5 + 0.5) as f32;
                                    let r = (34.0 + pulse * 50.0) as u8;
                                    let g = (197.0 + pulse * 30.0).min(255.0) as u8;
                                    let b = (94.0 + pulse * 50.0) as u8;
                                    egui::Color32::from_rgb(r, g, b)
                                } else {
                                    self.save_button_pulse_start = None;
                                    egui::Color32::from_rgb(34, 197, 94)
                                }
                            } else {
                                egui::Color32::from_rgb(34, 197, 94)
                            };

                            if ui.add(egui::Button::new(egui::RichText::new("💾 Save Now").size(13.0))
                                .fill(button_color)
                                .min_size(egui::vec2(100.0, 28.0)))
                                .on_hover_text("Save results to CSV (Ctrl+S)")
                                .clicked() {
                                self.save_csv();
                            }

                            // Open folder / open file buttons (only shown when a save path is set)
                            if !self.csv_output_path.is_empty() {
                                if ui.add(egui::Button::new(egui::RichText::new("📂 Open Folder").size(13.0))
                                    .min_size(egui::vec2(100.0, 28.0)))
                                    .on_hover_text("Open the folder containing the output file")
                                    .clicked() {
                                    let folder = PathBuf::from(&self.csv_output_path)
                                        .parent()
                                        .map(|p| p.to_path_buf())
                                        .unwrap_or_else(|| PathBuf::from("."));
                                    let _ = opener::open(folder);
                                }

                                if ui.add(egui::Button::new(egui::RichText::new("📊 Open File").size(13.0))
                                    .min_size(egui::vec2(90.0, 28.0)))
                                    .on_hover_text("Open the output file with the default program")
                                    .clicked() {
                                    let _ = opener::open(PathBuf::from(&self.csv_output_path));
                                }
                            }

                            // Show result count
                            if let Some(proc) = self.processor.lock().unwrap().as_ref() {
                                ui.label(egui::RichText::new(format!("({} receipts ready)", proc.get_results().len()))
                                    .color(egui::Color32::GRAY).size(12.0));
                            }
                        });
                    }
                });

            ui.add_space(10.0);

            // Documents Section
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(30, 41, 59))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(15.0))
                .show(ui, |ui| {
                    // Header with collapsible toggle and Add Files button
                    ui.horizontal(|ui| {
                        // File count badge in header
                        let header_text = if self.selected_files.is_empty() {
                            "📁 Documents".to_string()
                        } else {
                            format!("📁 Documents ({})", self.selected_files.len())
                        };
                        let arrow = if self.documents_section_open { "▼" } else { "▶" };

                        ui.toggle_value(&mut self.documents_section_open,
                            egui::RichText::new(arrow).size(12.0));
                        ui.label(egui::RichText::new(header_text).strong().size(16.0));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(egui::RichText::new("+ Add Files").size(13.0))
                                .fill(egui::Color32::from_rgb(59, 130, 246))
                                .min_size(egui::vec2(90.0, 26.0)))
                                .on_hover_text("Select files to process (Ctrl+O)")
                                .clicked() {
                                self.add_more_files();
                            }
                        });
                    });

                    // Collapsible content
                    if self.documents_section_open {
                        ui.add_space(10.0);

                        if self.selected_files.is_empty() {
                        // Empty state with engaging illustration and drop hint
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(30, 41, 59))
                            .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(71, 85, 105)))
                            .rounding(egui::Rounding::same(12.0))
                            .inner_margin(egui::Margin::same(40.0))
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    // Animated receipt icon (using multiple emojis for visual interest)
                                    ui.label(egui::RichText::new("🧾 📑 🗂️").size(40.0));
                                    ui.add_space(12.0);

                                    ui.label(egui::RichText::new("No receipts loaded yet")
                                        .size(18.0)
                                        .strong()
                                        .color(egui::Color32::WHITE));
                                    ui.add_space(8.0);

                                    ui.label(egui::RichText::new("Get started by adding your receipt files")
                                        .size(13.0)
                                        .color(egui::Color32::from_rgb(148, 163, 184)));
                                    ui.add_space(16.0);

                                    // Action options - single centered label
                                    ui.label(egui::RichText::new("📁 Click '+ Add Files' above or press Ctrl+O")
                                        .size(12.0)
                                        .color(egui::Color32::LIGHT_GRAY));

                                    ui.add_space(12.0);

                                    // Supported formats
                                    ui.label(egui::RichText::new("Supported: PDF, PNG, JPG, TXT, HTML")
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(100, 116, 139)));

                                    // Keyboard shortcut hint
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new("⌨️ Ctrl+O to open files")
                                        .size(10.0)
                                        .color(egui::Color32::from_rgb(100, 116, 139)));
                                });
                            });
                    } else {
                        // File count badge
                        ui.horizontal(|ui| {
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(59, 130, 246))
                                .rounding(egui::Rounding::same(10.0))
                                .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(format!("{} file(s)", self.selected_files.len())).color(egui::Color32::WHITE).small());
                                });
                            ui.add_space(10.0);
                            if ui.small_button("🗑️ Clear All").clicked() {
                                self.selected_files.clear();
                            }
                        });

                        ui.add_space(8.0);

                        // File list with remove buttons
                        let mut file_to_remove: Option<usize> = None;
                        egui::ScrollArea::vertical()
                            .id_salt("documents_file_list")
                            .max_height(180.0)
                            .show(ui, |ui| {
                            for (idx, file) in self.selected_files.iter().enumerate() {
                                egui::Frame::none()
                                    .fill(egui::Color32::from_rgb(51, 65, 85))
                                    .rounding(egui::Rounding::same(6.0))
                                    .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            // File icon based on extension
                                            let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
                                            let icon = match ext.to_lowercase().as_str() {
                                                "pdf" => "P",
                                                "png" | "jpg" | "jpeg" => "I",
                                                "txt" => "T",
                                                "html" | "htm" => "H",
                                                _ => "?",
                                            };
                                            ui.label(icon);

                                            // File name (not full path)
                                            let filename = file.file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("Unknown");
                                            ui.label(filename);

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("✖").on_hover_text("Remove this file").clicked() {
                                                    file_to_remove = Some(idx);
                                                }
                                            });
                                        });
                                    });
                                ui.add_space(3.0);
                            }
                        });

                        if let Some(idx) = file_to_remove {
                            self.selected_files.remove(idx);
                        }
                    }
                    }  // End of documents_section_open
                });

            ui.add_space(15.0);

            // Extraction Fields Section (inline on main page)
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(30, 41, 59))
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(egui::Margin::same(15.0))
                .show(ui, |ui| {
                    // Header with collapsible toggle
                    let current_fields_count = if self.extraction_mode == ExtractionMode::General {
                        self.extraction_fields.len()
                    } else {
                        self.item_fields.len()
                    };
                    ui.horizontal(|ui| {
                        let arrow = if self.fields_section_open { "▼" } else { "▶" };
                        ui.toggle_value(&mut self.fields_section_open,
                            egui::RichText::new(arrow).size(12.0));
                        ui.label(egui::RichText::new(format!("📝 Extraction Fields ({})", current_fields_count)).strong().size(16.0));
                    });

                    // Collapsible content
                    if self.fields_section_open {

                    // Mode selection radio buttons
                    ui.add_space(4.0);
                    let mut mode_changed = false;
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Mode:").size(12.0));
                        ui.add_space(8.0);

                        // General mode radio button
                        let general_selected = self.extraction_mode == ExtractionMode::General;
                        if ui.radio(general_selected, "").clicked() && !general_selected {
                            self.extraction_mode = ExtractionMode::General;
                            mode_changed = true;
                        }
                        ui.label(egui::RichText::new("General").size(12.0));
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("(Summary per receipt)").size(10.0).color(egui::Color32::GRAY));

                        ui.add_space(16.0);

                        // Detailed mode radio button
                        let detailed_selected = self.extraction_mode == ExtractionMode::Detailed;
                        if ui.radio(detailed_selected, "").clicked() && !detailed_selected {
                            self.extraction_mode = ExtractionMode::Detailed;
                            mode_changed = true;
                        }
                        ui.label(egui::RichText::new("Detailed").size(12.0));
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("(Items per receipt)").size(10.0).color(egui::Color32::GRAY));
                    });

                    if mode_changed {
                        self.save_api_keys();
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Show appropriate fields based on mode
                    if self.extraction_mode == ExtractionMode::General {
                    // General extraction fields
                    ui.label(egui::RichText::new("Drag to reorder • Fields appear in CSV in this order").size(11.0).color(egui::Color32::GRAY));

                    ui.add_space(8.0);

                    // Field chips with drag-and-drop reordering
                    let mut to_remove: Option<usize> = None;
                    let mut drop_target: Option<usize> = None;

                    // Calculate hover progress for visual feedback (0.0 to 1.0)
                    let hover_progress: f32 = if let (Some(_), Some(start)) = (self.drag_hover_target, self.drag_hover_start) {
                        (start.elapsed().as_secs_f32() / 1.5).min(1.0)
                    } else {
                        0.0
                    };

                    // Calculate drop animation progress with easing (0.0 to 1.0 over 0.6 seconds)
                    let (anim_from, anim_to, anim_progress) = if let Some((from, to, start)) = self.field_drop_animation {
                        let elapsed = start.elapsed().as_secs_f32();
                        let duration = 0.6; // Longer duration for smoother feel
                        if elapsed > duration {
                            self.field_drop_animation = None;
                            (None, None, 0.0)
                        } else {
                            // Request repaint for smooth animation
                            ui.ctx().request_repaint();
                            // Ease-out cubic: starts fast, slows down at end (feels more natural)
                            let t = elapsed / duration;
                            let eased = 1.0 - (1.0 - t).powi(3);
                            (Some(from), Some(to), 1.0 - eased) // Fade out from 1.0 to 0.0
                        }
                    } else {
                        (None, None, 0.0)
                    };

                    egui::ScrollArea::horizontal()
                        .id_salt("extraction_fields_scroll")
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    let is_dragging = self.dragging_field_idx.is_some();

                                    for (idx, field) in self.extraction_fields.iter().enumerate() {
                                        let is_being_dragged = self.dragging_field_idx == Some(idx);
                                        let is_drop_target = drop_target == Some(idx);
                                        let is_hover_target = self.drag_hover_target == Some(idx);

                                        // Check if this item is part of the drop animation
                                        // The dropped item (now at to_idx) gets a "pop" effect
                                        // Items that were bumped (between from and to) get a subtle highlight
                                        let is_dropped_item = anim_to == Some(idx);
                                        let is_bumped_item = if let (Some(from), Some(to)) = (anim_from, anim_to) {
                                            if from < to {
                                                // Moved right: items from+1 to to were bumped left
                                                idx > from && idx < to
                                            } else {
                                                // Moved left: items from to to+1 were bumped right
                                                idx >= to && idx < from
                                            }
                                        } else {
                                            false
                                        };

                                        // Show drop indicator BEFORE this item if hovering here
                                        // and we're dragging from a position after this one
                                        if is_dragging && is_hover_target {
                                            if let Some(from_idx) = self.dragging_field_idx {
                                                if from_idx > idx {
                                                    // Animated drop indicator - pulses based on hover progress
                                                    let pulse = (hover_progress * std::f32::consts::PI * 4.0).sin() * 0.3 + 0.7;
                                                    let alpha = (200.0 * pulse) as u8;

                                                    // Wide gap with animated line
                                                    ui.add_space(4.0);
                                                    let indicator_rect = egui::Rect::from_min_size(
                                                        ui.cursor().min,
                                                        egui::Vec2::new(4.0, 32.0) // Thicker and taller
                                                    );
                                                    ui.painter().rect_filled(
                                                        indicator_rect,
                                                        egui::Rounding::same(2.0),
                                                        egui::Color32::from_rgba_unmultiplied(34, 197, 94, alpha) // Green with pulse
                                                    );
                                                    ui.add_space(8.0);
                                                }
                                            }
                                        }

                                        // Choose color based on drag state with hover feedback and animation
                                        let bg_color = if is_dropped_item && anim_progress > 0.0 {
                                            // Dropped item: bright green highlight that fades to normal blue
                                            let green_r = 34;
                                            let green_g = 197;
                                            let green_b = 94;
                                            let blue_r = 59;
                                            let blue_g = 130;
                                            let blue_b = 246;
                                            let r = (blue_r as f32 + (green_r - blue_r) as f32 * anim_progress) as u8;
                                            let g = (blue_g as f32 + (green_g - blue_g) as f32 * anim_progress) as u8;
                                            let b = (blue_b as f32 + (green_b - blue_b) as f32 * anim_progress) as u8;
                                            egui::Color32::from_rgb(r, g, b)
                                        } else if is_bumped_item && anim_progress > 0.0 {
                                            // Bumped items: subtle gold highlight that fades to normal blue
                                            let gold_r = 251;
                                            let gold_g = 191;
                                            let gold_b = 36;
                                            let blue_r = 59;
                                            let blue_g = 130;
                                            let blue_b = 246;
                                            let intensity = anim_progress * 0.5; // Subtle effect
                                            let r = (blue_r as f32 + (gold_r - blue_r) as f32 * intensity) as u8;
                                            let g = (blue_g as f32 + (gold_g - blue_g) as f32 * intensity) as u8;
                                            let b = (blue_b as f32 + (gold_b - blue_b) as f32 * intensity) as u8;
                                            egui::Color32::from_rgb(r, g, b)
                                        } else if is_being_dragged {
                                            // Dragged item: semi-transparent with dashed border effect
                                            egui::Color32::from_rgba_unmultiplied(59, 130, 246, 100)
                                        } else if is_hover_target && hover_progress > 0.0 {
                                            // Interpolate from highlight to green as progress increases
                                            let r = (99.0 + (34.0 - 99.0) * hover_progress) as u8;
                                            let g = (170.0 + (197.0 - 170.0) * hover_progress) as u8;
                                            let b = (255.0 + (94.0 - 255.0) * hover_progress) as u8;
                                            egui::Color32::from_rgb(r, g, b)
                                        } else if is_drop_target {
                                            egui::Color32::from_rgb(99, 170, 255) // Highlight drop target
                                        } else {
                                            egui::Color32::from_rgb(59, 130, 246)
                                        };

                                        // Stroke: animation effects, dashed effect for dragged, gold for drop target
                                        let stroke = if is_dropped_item && anim_progress > 0.0 {
                                            // Dropped item: green border that fades out
                                            let alpha = (255.0 * anim_progress) as u8;
                                            egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(34, 197, 94, alpha))
                                        } else if is_bumped_item && anim_progress > 0.0 {
                                            // Bumped items: subtle gold border that fades out
                                            let alpha = (180.0 * anim_progress) as u8;
                                            egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(251, 191, 36, alpha))
                                        } else if is_being_dragged {
                                            egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 200, 200)) // Gray dashed-like border
                                        } else if is_hover_target && hover_progress > 0.0 {
                                            let thickness = 2.0 + hover_progress * 2.0;
                                            egui::Stroke::new(thickness, egui::Color32::from_rgb(251, 191, 36))
                                        } else if is_drop_target {
                                            egui::Stroke::new(2.0, egui::Color32::from_rgb(251, 191, 36))
                                        } else {
                                            egui::Stroke::NONE
                                        };

                                        // Track drag handle response for drag-only on handle
                                        let mut drag_handle_response: Option<egui::Response> = None;

                                        let response = egui::Frame::none()
                                            .fill(bg_color)
                                            .rounding(egui::Rounding::same(8.0))
                                            .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                                            .stroke(stroke)
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    let text_color = if is_being_dragged {
                                                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 150)
                                                    } else {
                                                        egui::Color32::WHITE
                                                    };
                                                    let handle_color = if is_being_dragged {
                                                        egui::Color32::from_rgba_unmultiplied(200, 200, 200, 150)
                                                    } else {
                                                        egui::Color32::from_rgb(100, 116, 139)
                                                    };

                                                    // Drag handle - this is the only draggable part
                                                    let handle = ui.add(
                                                        egui::Label::new(egui::RichText::new("⋮⋮").color(handle_color).size(12.0))
                                                            .sense(egui::Sense::drag())
                                                    );
                                                    if handle.hovered() && !is_dragging {
                                                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                                                    }
                                                    if handle.dragged() {
                                                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                                                    }
                                                    drag_handle_response = Some(handle);

                                                    ui.add_space(4.0);

                                                    // Field name with enhancement status indicator
                                                    let display_name = if field.is_enhancing {
                                                        format!("⏳ {}", field.name)  // Spinner while enhancing
                                                    } else if field.extraction_prompt.is_some() {
                                                        format!("✨ {}", field.name)  // Enhanced
                                                    } else {
                                                        field.name.clone()
                                                    };
                                                    let label = ui.label(egui::RichText::new(&display_name).color(text_color).size(12.0));

                                                    // Show enhanced prompt as tooltip
                                                    if let Some(prompt) = &field.extraction_prompt {
                                                        label.on_hover_text(format!("Extraction prompt: {}", prompt));
                                                    }

                                                    // Remove button (hidden when dragging) - larger hit area
                                                    if !is_being_dragged {
                                                        ui.add_space(6.0); // Add spacing before X button
                                                        let x_btn = ui.add(egui::Button::new(egui::RichText::new("✖").size(12.0))
                                                            .fill(egui::Color32::from_rgb(71, 85, 105))
                                                            .rounding(egui::Rounding::same(4.0))
                                                            .min_size(egui::vec2(22.0, 22.0)))
                                                            .on_hover_text("Remove field");
                                                        if x_btn.clicked() {
                                                            to_remove = Some(idx);
                                                        }
                                                    }
                                                });
                                            }).response;

                                        // Handle drag start - only from the drag handle
                                        if let Some(handle) = drag_handle_response {
                                            if handle.drag_started() {
                                                self.dragging_field_idx = Some(idx);
                                                self.dragging_field_text = Some(field.name.clone());
                                                self.drag_hover_target = None;
                                                self.drag_hover_start = None;
                                            }
                                        }

                                        // Handle drop target detection - use rect.contains for drag detection
                                        // response.hovered() doesn't work during drag, so check pointer position manually
                                        let is_pointer_over = if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                            response.rect.contains(pointer_pos)
                                        } else {
                                            false
                                        };

                                        if is_dragging && !is_being_dragged && is_pointer_over {
                                            drop_target = Some(idx);
                                            // Update hover target immediately so indicator shows on this frame
                                            if self.drag_hover_target != Some(idx) {
                                                self.drag_hover_target = Some(idx);
                                                self.drag_hover_start = Some(std::time::Instant::now());
                                            }
                                        }

                                        ui.add_space(4.0);

                                        // Show drop indicator AFTER this item if currently hovering here
                                        // and we're dragging from a position before this one
                                        let show_indicator_after = is_dragging &&
                                            self.drag_hover_target == Some(idx) &&
                                            self.dragging_field_idx.map_or(false, |from| from < idx);

                                        if show_indicator_after {
                                            // Animated drop indicator - pulses based on hover progress
                                            let pulse = (hover_progress * std::f32::consts::PI * 4.0).sin() * 0.3 + 0.7;
                                            let alpha = (200.0 * pulse) as u8;

                                            // Wide gap with animated line
                                            ui.add_space(4.0);
                                            let indicator_rect = egui::Rect::from_min_size(
                                                ui.cursor().min,
                                                egui::Vec2::new(4.0, 32.0) // Thicker and taller
                                            );
                                            ui.painter().rect_filled(
                                                indicator_rect,
                                                egui::Rounding::same(2.0),
                                                egui::Color32::from_rgba_unmultiplied(34, 197, 94, alpha) // Green with pulse
                                            );
                                            ui.add_space(8.0);
                                        }
                                    }
                                });
                                ui.add_space(8.0);
                            });
                        });

                    // Handle hover timing for auto-insert after 1.5 seconds
                    let mut fields_changed = false;
                    let mut auto_insert_triggered = false;

                    if let Some(from_idx) = self.dragging_field_idx {
                        if let Some(to_idx) = drop_target {
                            // Check if we're hovering over a new target
                            if self.drag_hover_target != Some(to_idx) {
                                // New target - start timing
                                self.drag_hover_target = Some(to_idx);
                                self.drag_hover_start = Some(std::time::Instant::now());
                            } else if let Some(start_time) = self.drag_hover_start {
                                // Same target - check if 1.5 seconds have passed
                                if start_time.elapsed().as_secs_f32() >= 1.5 && from_idx != to_idx {
                                    // Auto-insert!
                                    let field = self.extraction_fields.remove(from_idx);
                                    let insert_idx = if to_idx > from_idx { to_idx } else { to_idx };
                                    self.extraction_fields.insert(insert_idx, field);
                                    fields_changed = true;
                                    auto_insert_triggered = true;
                                    // Start drop animation
                                    self.field_drop_animation = Some((from_idx, insert_idx, std::time::Instant::now()));
                                    // Reset drag state
                                    self.dragging_field_idx = None;
                                    self.dragging_field_text = None;
                                    self.drag_hover_target = None;
                                    self.drag_hover_start = None;
                                }
                            }
                        } else {
                            // Not hovering over any target - reset timing
                            self.drag_hover_target = None;
                            self.drag_hover_start = None;
                        }
                    }

                    // Handle manual drag end (mouse release)
                    if !auto_insert_triggered && ui.input(|i| i.pointer.any_released()) {
                        if let (Some(from), Some(to)) = (self.dragging_field_idx, drop_target) {
                            if from != to {
                                // Remove from old position and insert at new position
                                let field = self.extraction_fields.remove(from);
                                self.extraction_fields.insert(to, field);
                                fields_changed = true;
                                // Start drop animation
                                self.field_drop_animation = Some((from, to, std::time::Instant::now()));
                            }
                        }
                        self.dragging_field_idx = None;
                        self.dragging_field_text = None;
                        self.drag_hover_target = None;
                        self.drag_hover_start = None;
                    }

                    // Draw floating ghost chip at cursor position
                    if let (Some(_idx), Some(ref text)) = (self.dragging_field_idx, &self.dragging_field_text) {
                        if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                            egui::Area::new(egui::Id::new("extraction_field_ghost"))
                                .fixed_pos(pointer_pos + egui::vec2(10.0, 10.0))
                                .order(egui::Order::Tooltip)
                                .show(ui.ctx(), |ui| {
                                    egui::Frame::none()
                                        .fill(egui::Color32::from_rgb(59, 130, 246))
                                        .rounding(egui::Rounding::same(8.0))
                                        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                                        .shadow(egui::epaint::Shadow {
                                            offset: egui::vec2(2.0, 4.0),
                                            blur: 8.0,
                                            spread: 0.0,
                                            color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 80),
                                        })
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("::").color(egui::Color32::from_rgb(200, 200, 200)).size(12.0));
                                                ui.label(egui::RichText::new(text).color(egui::Color32::WHITE).size(12.0));
                                            });
                                        });
                                });
                        }
                    }

                    // Request repaint while dragging to update hover timer and ghost position
                    if self.dragging_field_idx.is_some() {
                        ui.ctx().request_repaint();
                    }

                    if let Some(idx) = to_remove {
                        self.extraction_fields.remove(idx);
                        fields_changed = true;
                    }

                    ui.add_space(10.0);

                    // Add new field inline
                    ui.horizontal(|ui| {
                        let text_edit = egui::TextEdit::singleline(&mut self.new_field_input)
                            .hint_text("Add custom field...")
                            .desired_width(200.0);
                        ui.add(text_edit);

                        let can_add = !self.new_field_input.trim().is_empty();
                        if ui.add_enabled(can_add, egui::Button::new(egui::RichText::new("+ Add").size(12.0))
                            .min_size(egui::vec2(55.0, 24.0)))
                            .clicked() {
                            let field_name = self.new_field_input.trim().to_string();
                            // Check if it's a known default field
                            let is_known = is_default_field(&field_name, DEFAULT_EXTRACTION_FIELDS);
                            let new_field = if is_known {
                                FieldDefinition::new_default(field_name.clone())
                            } else {
                                // Custom field - will trigger background enhancement
                                let mut f = FieldDefinition::new_custom(field_name.clone());
                                f.is_enhancing = true;  // Mark as enhancing
                                f
                            };

                            // Trigger background prompt enhancement for custom fields
                            if !is_known {
                                self.trigger_field_enhancement(field_name, self.extraction_fields.len(), false);
                            }

                            self.extraction_fields.push(new_field);
                            self.new_field_input.clear();
                            fields_changed = true;
                        }
                    });

                    // Save fields whenever they change
                    if fields_changed {
                        save_extraction_fields(&self.extraction_fields);
                    }
                    } else {
                    // Detailed extraction (item fields)
                    ui.label(egui::RichText::new("Drag to reorder • Fields to extract for each item").size(11.0).color(egui::Color32::GRAY));

                    ui.add_space(8.0);

                    // Item field chips with drag-and-drop reordering
                    let mut item_to_remove: Option<usize> = None;
                    let mut item_drop_target: Option<usize> = None;

                    // Calculate hover progress for visual feedback (0.0 to 1.0)
                    let item_hover_progress: f32 = if let (Some(_), Some(start)) = (self.item_drag_hover_target, self.item_drag_hover_start) {
                        (start.elapsed().as_secs_f32() / 1.5).min(1.0)
                    } else {
                        0.0
                    };

                    // Calculate drop animation progress with easing (0.0 to 1.0 over 0.6 seconds)
                    let (item_anim_from, item_anim_to, item_anim_progress) = if let Some((from, to, start)) = self.item_field_drop_animation {
                        let elapsed = start.elapsed().as_secs_f32();
                        let duration = 0.6;
                        if elapsed > duration {
                            self.item_field_drop_animation = None;
                            (None, None, 0.0)
                        } else {
                            ui.ctx().request_repaint();
                            let t = elapsed / duration;
                            let eased = 1.0 - (1.0 - t).powi(3);
                            (Some(from), Some(to), 1.0 - eased)
                        }
                    } else {
                        (None, None, 0.0)
                    };

                    let mut item_fields_changed = false;

                    egui::ScrollArea::horizontal()
                        .id_salt("item_fields_scroll")
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    let is_dragging = self.dragging_item_field_idx.is_some();

                                    for (idx, field) in self.item_fields.iter().enumerate() {
                                        let is_being_dragged = self.dragging_item_field_idx == Some(idx);
                                        let is_hover_target = self.item_drag_hover_target == Some(idx);

                                        // Check if this item is part of the drop animation
                                        let is_dropped_item = item_anim_to == Some(idx);
                                        let is_bumped_item = if let (Some(from), Some(to)) = (item_anim_from, item_anim_to) {
                                            if from < to {
                                                idx > from && idx < to
                                            } else {
                                                idx >= to && idx < from
                                            }
                                        } else {
                                            false
                                        };

                                        // Show drop indicator BEFORE this item if hovering here
                                        if is_dragging && is_hover_target {
                                            if let Some(from_idx) = self.dragging_item_field_idx {
                                                if from_idx > idx {
                                                    // Draw pulsing indicator
                                                    let pulse = (ui.ctx().input(|i| i.time) * 3.0).sin() * 0.3 + 0.7;
                                                    let indicator_color = egui::Color32::from_rgba_unmultiplied(34, 197, 94, (pulse * 255.0) as u8);
                                                    let (rect, _) = ui.allocate_exact_size(egui::vec2(4.0, 32.0), egui::Sense::hover());
                                                    ui.painter().rect_filled(rect, 2.0, indicator_color);
                                                    ui.add_space(2.0);
                                                }
                                            }
                                        }

                                        // Base colors for item fields (purple theme)
                                        let base_fill = egui::Color32::from_rgb(88, 28, 135);
                                        let base_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(168, 85, 247));

                                        // Determine fill and stroke based on state
                                        let (fill_color, stroke) = if is_being_dragged {
                                            (egui::Color32::from_rgba_unmultiplied(88, 28, 135, 100), egui::Stroke::new(1.0, egui::Color32::GRAY))
                                        } else if is_dropped_item && item_anim_progress > 0.0 {
                                            let green = egui::Color32::from_rgb(34, 197, 94);
                                            let r = ((green.r() as f32 * item_anim_progress + base_fill.r() as f32 * (1.0 - item_anim_progress))) as u8;
                                            let g = ((green.g() as f32 * item_anim_progress + base_fill.g() as f32 * (1.0 - item_anim_progress))) as u8;
                                            let b = ((green.b() as f32 * item_anim_progress + base_fill.b() as f32 * (1.0 - item_anim_progress))) as u8;
                                            (egui::Color32::from_rgb(r, g, b), egui::Stroke::new(2.0, egui::Color32::from_rgb(34, 197, 94)))
                                        } else if is_bumped_item && item_anim_progress > 0.0 {
                                            let gold = egui::Color32::from_rgb(234, 179, 8);
                                            let blend = item_anim_progress * 0.3;
                                            let r = ((gold.r() as f32 * blend + base_fill.r() as f32 * (1.0 - blend))) as u8;
                                            let g = ((gold.g() as f32 * blend + base_fill.g() as f32 * (1.0 - blend))) as u8;
                                            let b = ((gold.b() as f32 * blend + base_fill.b() as f32 * (1.0 - blend))) as u8;
                                            (egui::Color32::from_rgb(r, g, b), egui::Stroke::new(1.0, egui::Color32::from_rgb(234, 179, 8)))
                                        } else if is_hover_target && is_dragging {
                                            let blue = egui::Color32::from_rgb(59, 130, 246);
                                            let green = egui::Color32::from_rgb(34, 197, 94);
                                            let r = ((blue.r() as f32 * (1.0 - item_hover_progress) + green.r() as f32 * item_hover_progress)) as u8;
                                            let g = ((blue.g() as f32 * (1.0 - item_hover_progress) + green.g() as f32 * item_hover_progress)) as u8;
                                            let b = ((blue.b() as f32 * (1.0 - item_hover_progress) + green.b() as f32 * item_hover_progress)) as u8;
                                            let stroke_width = 2.0 + item_hover_progress * 2.0;
                                            (egui::Color32::from_rgb(r, g, b), egui::Stroke::new(stroke_width, egui::Color32::from_rgb(r, g, b)))
                                        } else {
                                            (base_fill, base_stroke)
                                        };

                                        let chip_frame = egui::Frame::none()
                                            .fill(fill_color)
                                            .stroke(stroke)
                                            .rounding(egui::Rounding::same(16.0))
                                            .inner_margin(egui::Margin::symmetric(10.0, 6.0));

                                        // Track drag handle response for drag-only on handle
                                        let mut item_drag_handle_response: Option<egui::Response> = None;

                                        let response = chip_frame.show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                let handle_color = if is_being_dragged {
                                                    egui::Color32::from_rgba_unmultiplied(200, 200, 200, 150)
                                                } else {
                                                    egui::Color32::from_rgb(180, 180, 220)
                                                };

                                                // Drag handle - this is the only draggable part
                                                let handle = ui.add(
                                                    egui::Label::new(egui::RichText::new("⋮⋮").color(handle_color).size(12.0))
                                                        .sense(egui::Sense::drag())
                                                );
                                                if handle.hovered() && !is_dragging {
                                                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                                                }
                                                if handle.dragged() {
                                                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                                                }
                                                item_drag_handle_response = Some(handle);

                                                ui.add_space(4.0);

                                                // Field name with enhancement status indicator
                                                let display_name = if field.is_enhancing {
                                                    format!("⏳ {}", field.name)  // Spinner while enhancing
                                                } else if field.extraction_prompt.is_some() {
                                                    format!("✨ {}", field.name)  // Enhanced
                                                } else {
                                                    field.name.clone()
                                                };
                                                let label = ui.label(egui::RichText::new(&display_name).size(12.0).color(egui::Color32::WHITE));

                                                // Show enhanced prompt as tooltip
                                                if let Some(prompt) = &field.extraction_prompt {
                                                    label.on_hover_text(format!("Extraction prompt: {}", prompt));
                                                }

                                                ui.add_space(6.0); // Add spacing before X button
                                                let x_btn = ui.add(egui::Button::new(egui::RichText::new("✖").size(12.0))
                                                    .fill(egui::Color32::from_rgb(100, 80, 140))
                                                    .rounding(egui::Rounding::same(4.0))
                                                    .min_size(egui::vec2(22.0, 22.0)))
                                                    .on_hover_text("Remove field");
                                                if x_btn.clicked() {
                                                    item_to_remove = Some(idx);
                                                }
                                            });
                                        }).response;

                                        // Handle drag start - only from the drag handle
                                        if let Some(handle) = item_drag_handle_response {
                                            if handle.drag_started() {
                                                self.dragging_item_field_idx = Some(idx);
                                                self.dragging_item_field_text = Some(field.name.clone());
                                            }
                                        }

                                        // Manual hover detection during drag
                                        if is_dragging && !is_being_dragged {
                                            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                                if response.rect.contains(pointer_pos) {
                                                    if self.item_drag_hover_target != Some(idx) {
                                                        self.item_drag_hover_target = Some(idx);
                                                        self.item_drag_hover_start = Some(std::time::Instant::now());
                                                    } else if let Some(start) = self.item_drag_hover_start {
                                                        if start.elapsed().as_secs_f32() >= 1.5 {
                                                            item_drop_target = Some(idx);
                                                        }
                                                    }
                                                    ui.ctx().request_repaint();
                                                }
                                            }
                                        }

                                        // Show drop indicator AFTER this item
                                        if is_dragging && is_hover_target {
                                            if let Some(from_idx) = self.dragging_item_field_idx {
                                                if from_idx < idx {
                                                    ui.add_space(2.0);
                                                    let pulse = (ui.ctx().input(|i| i.time) * 3.0).sin() * 0.3 + 0.7;
                                                    let indicator_color = egui::Color32::from_rgba_unmultiplied(34, 197, 94, (pulse * 255.0) as u8);
                                                    let (rect, _) = ui.allocate_exact_size(egui::vec2(4.0, 32.0), egui::Sense::hover());
                                                    ui.painter().rect_filled(rect, 2.0, indicator_color);
                                                }
                                            }
                                        }

                                        ui.add_space(4.0);
                                    }

                                    // Handle drag end
                                    if ui.input(|i| i.pointer.any_released()) && self.dragging_item_field_idx.is_some() {
                                        self.dragging_item_field_idx = None;
                                        self.dragging_item_field_text = None;
                                        self.item_drag_hover_target = None;
                                        self.item_drag_hover_start = None;
                                    }
                                });

                                // Floating ghost chip
                                if let (Some(_from_idx), Some(ref text)) = (self.dragging_item_field_idx, &self.dragging_item_field_text) {
                                    if let Some(pos) = ui.ctx().pointer_interact_pos() {
                                        egui::Area::new(egui::Id::new("item_field_drag_ghost"))
                                            .fixed_pos(pos + egui::vec2(10.0, 10.0))
                                            .order(egui::Order::Tooltip)
                                            .show(ui.ctx(), |ui| {
                                                egui::Frame::none()
                                                    .fill(egui::Color32::from_rgba_unmultiplied(168, 85, 247, 200))
                                                    .rounding(egui::Rounding::same(16.0))
                                                    .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                                                    .show(ui, |ui| {
                                                        ui.label(egui::RichText::new(text).size(12.0).color(egui::Color32::WHITE));
                                                    });
                                            });
                                    }
                                }
                            });
                        });

                    // Handle drop (auto-insert after hover timeout)
                    if let (Some(from_idx), Some(to_idx)) = (self.dragging_item_field_idx, item_drop_target) {
                        if from_idx != to_idx {
                            let field = self.item_fields.remove(from_idx);
                            let insert_idx = if from_idx < to_idx { to_idx } else { to_idx };
                            self.item_fields.insert(insert_idx, field);
                            self.item_field_drop_animation = Some((from_idx, insert_idx, std::time::Instant::now()));
                            item_fields_changed = true;
                        }
                        self.dragging_item_field_idx = None;
                        self.dragging_item_field_text = None;
                        self.item_drag_hover_target = None;
                        self.item_drag_hover_start = None;
                    }

                    if let Some(idx) = item_to_remove {
                        self.item_fields.remove(idx);
                        item_fields_changed = true;
                    }

                    ui.add_space(10.0);

                    // Add new item field inline
                    ui.horizontal(|ui| {
                        let text_edit = egui::TextEdit::singleline(&mut self.new_item_field_input)
                            .hint_text("Add custom item field...")
                            .desired_width(200.0);
                        ui.add(text_edit);

                        let can_add = !self.new_item_field_input.trim().is_empty();
                        if ui.add_enabled(can_add, egui::Button::new(egui::RichText::new("+ Add").size(12.0))
                            .min_size(egui::vec2(55.0, 24.0)))
                            .clicked() {
                            let field_name = self.new_item_field_input.trim().to_string();
                            // Check if it's a known default field
                            let is_known = is_default_field(&field_name, DEFAULT_ITEM_FIELDS);
                            let new_field = if is_known {
                                FieldDefinition::new_default(field_name.clone())
                            } else {
                                // Custom field - will trigger background enhancement
                                let mut f = FieldDefinition::new_custom(field_name.clone());
                                f.is_enhancing = true;  // Mark as enhancing
                                f
                            };

                            // Trigger background prompt enhancement for custom fields
                            if !is_known {
                                self.trigger_field_enhancement(field_name, self.item_fields.len(), true);
                            }

                            self.item_fields.push(new_field);
                            self.new_item_field_input.clear();
                            item_fields_changed = true;
                        }
                    });

                    // Save item fields whenever they change
                    if item_fields_changed {
                        save_item_fields(&self.item_fields);
                    }
                    } // End of mode conditional (General/Detailed)

                    }  // End of fields_section_open
                });

            ui.add_space(12.0);

            // Rate Limit Warning (if detected during pre-check)
            if let Some(warning) = &self.rate_limit_warning {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(120, 53, 15))  // Orange/amber background
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("!").size(20.0).color(egui::Color32::from_rgb(251, 191, 36)));
                            ui.add_space(8.0);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Rate Limit Detected").strong().color(egui::Color32::from_rgb(254, 215, 170)));
                                ui.add_space(4.0);
                                ui.label(egui::RichText::new(warning).color(egui::Color32::from_rgb(253, 230, 138)));
                            });
                        });
                    });
                ui.add_space(8.0);
            }

            // Status Messages Card
            if self.state == AppState::Processing {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(30, 58, 138))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(15.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(egui::RichText::new(&self.progress_message).strong());
                        });

                        // Show countdown timer if rate limited
                        if let Some(countdown) = self.rate_limit_countdown {
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⏳").size(16.0).color(egui::Color32::from_rgb(59, 130, 246)));
                                ui.add_space(8.0);
                                ui.label(egui::RichText::new(format!("Waiting {} seconds...", countdown))
                                    .strong()
                                    .color(egui::Color32::from_rgb(251, 191, 36)));
                            });
                        }

                        ui.add_space(8.0);
                        if self.total_progress > 0 {
                            let progress = self.current_progress as f32 / self.total_progress as f32;
                            ui.add(egui::ProgressBar::new(progress).show_percentage());
                        }
                    });
            }

            if self.state == AppState::ReadyToSave {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(20, 83, 45))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(15.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("✓ Processing complete!").strong().color(egui::Color32::from_rgb(134, 239, 172)));
                        ui.add_space(5.0);
                        ui.label("Use the 'CSV Output' section above to save your data.");
                    });

                // Show warning for incomplete extractions
                if !self.incomplete_extractions.is_empty() {
                    ui.add_space(10.0);
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(120, 80, 20))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(format!(
                                "! {} receipt(s) have missing data:",
                                self.incomplete_extractions.len()
                            )).strong().color(egui::Color32::from_rgb(253, 224, 71)));
                            ui.add_space(6.0);

                            egui::ScrollArea::vertical()
                                .id_salt("incomplete_extractions_list")
                                .max_height(150.0)
                                .show(ui, |ui| {
                                    for incomplete in &self.incomplete_extractions {
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("•").color(egui::Color32::from_rgb(253, 224, 71)));
                                                ui.label(egui::RichText::new(&incomplete.file_name).strong());
                                            });
                                            // Show error reason if available
                                            if let Some(error) = &incomplete.error_reason {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(12.0);
                                                    ui.label(egui::RichText::new(format!("Reason: {}", error))
                                                        .color(egui::Color32::from_rgb(252, 165, 165))
                                                        .size(11.0));
                                                });
                                            }
                                            // Show missing fields if any
                                            if !incomplete.missing_fields.is_empty() {
                                                ui.horizontal(|ui| {
                                                    ui.add_space(12.0);
                                                    ui.label(egui::RichText::new(format!("Missing: {}", incomplete.missing_fields.join(", ")))
                                                        .size(11.0));
                                                });
                                            }
                                            ui.add_space(4.0);
                                        });
                                    }
                                });

                            ui.add_space(6.0);
                            ui.label(egui::RichText::new("Tip: These files may be low quality, have truncated responses, or hit rate limits.").size(11.0).italics());
                        });
                }
            }

            if self.state == AppState::Error {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(127, 29, 29))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(15.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(format!("✗ Error: {}", self.error_message)).color(egui::Color32::from_rgb(252, 165, 165)));
                        ui.add_space(8.0);
                        if ui.add(egui::Button::new(egui::RichText::new("Dismiss").size(13.0))
                            .min_size(egui::vec2(80.0, 28.0)))
                            .clicked() {
                            self.state = AppState::Idle;
                            self.error_message.clear();
                        }
                    });
            }

            if self.state == AppState::Completed {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(20, 83, 45))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(15.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("✓ Extraction completed!").strong().size(16.0).color(egui::Color32::from_rgb(134, 239, 172)));
                        ui.add_space(5.0);
                        if let Some(path) = &self.output_path {
                            ui.label(format!("Saved to: {}", path.display()));
                        }
                    });

                // Buttons outside the frame to avoid borrow issues
                let parent_path = self.output_path.as_ref().and_then(|p| p.parent().map(|pp| pp.to_path_buf()));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(egui::RichText::new("Open Folder").size(13.0))
                        .min_size(egui::vec2(100.0, 28.0)))
                        .clicked() {
                        if let Some(parent) = &parent_path {
                            let _ = opener::open(parent);
                        }
                    }
                    ui.add_space(8.0);
                    if ui.add(egui::Button::new(egui::RichText::new("↻ New Extraction").size(13.0))
                        .min_size(egui::vec2(130.0, 28.0)))
                        .clicked() {
                        self.state = AppState::Idle;
                        self.selected_files.clear();
                        self.output_path = None;
                        self.incomplete_extractions.clear();
                    }
                });
            }

            // Bottom padding to ensure content doesn't get cut off by status bar
            ui.add_space(30.0);

            }); // End of main ScrollArea
        });

        // Tip of the Day Modal Window (overlay)
        if self.show_tip_widget {
            let tips = get_tips();
            let (tip_title, tip_text) = tips[self.current_tip_index % tips.len()];

            egui::Window::new("💡 Tip of the Day")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .fixed_size([500.0, 200.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);

                        // Tip title
                        ui.label(egui::RichText::new(tip_title)
                            .strong()
                            .size(18.0)
                            .color(egui::Color32::from_rgb(59, 130, 246)));

                        ui.add_space(15.0);

                        // Tip content
                        ui.label(egui::RichText::new(tip_text)
                            .size(14.0)
                            .color(egui::Color32::LIGHT_GRAY));

                        ui.add_space(20.0);
                    });

                    // Bottom section with navigation and controls
                    ui.separator();
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        // Left side - navigation
                        if ui.add(egui::Button::new(egui::RichText::new("< Prev").size(12.0))
                            .min_size(egui::vec2(60.0, 24.0)))
                            .clicked() {
                            self.current_tip_index = if self.current_tip_index == 0 {
                                tips.len() - 1
                            } else {
                                self.current_tip_index - 1
                            };
                        }

                        ui.label(egui::RichText::new(format!("{}/{}",
                            self.current_tip_index + 1,
                            tips.len()))
                            .size(11.0)
                            .color(egui::Color32::GRAY));

                        if ui.add(egui::Button::new(egui::RichText::new("Next >").size(12.0))
                            .min_size(egui::vec2(60.0, 24.0)))
                            .clicked() {
                            self.current_tip_index = (self.current_tip_index + 1) % tips.len();
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Close button
                            if ui.add(egui::Button::new(egui::RichText::new("Close").size(12.0))
                                .min_size(egui::vec2(55.0, 24.0)))
                                .clicked() {
                                self.show_tip_widget = false;
                            }

                            // Show on startup checkbox
                            if ui.checkbox(&mut self.show_tips_on_startup, "Show on startup")
                                .changed() {
                                self.save_api_keys();
                            }
                        });
                    });
                });
        }

        // Help Modal Window (Getting Started Guide)
        if self.show_help_modal {
            // Get available screen size and constrain modal to fit
            let screen_rect = ctx.screen_rect();
            let max_width = (screen_rect.width() - 40.0).min(700.0);
            let max_height = (screen_rect.height() - 80.0).min(500.0);

            egui::Window::new("Getting Started Guide")
                .collapsible(false)
                .resizable(true)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .default_size([max_width, max_height])
                .max_size([max_width, max_height])
                .show(ctx, |ui| {
                    // Calculate scroll area height to leave room for Close button
                    let available_height = ui.available_height() - 40.0;

                    egui::ScrollArea::vertical()
                        .max_height(available_height)
                        .show(ui, |ui| {
                            // How to Use the App
                            ui.label(egui::RichText::new("How to Use This App").strong().size(16.0).color(egui::Color32::from_rgb(59, 130, 246)));
                            ui.add_space(10.0);

                            ui.label(egui::RichText::new("Step 1: Set Up Your API Key").strong());
                            ui.label("  1. Choose an AI provider from the dropdown in Settings (Groq, OpenAI, Claude, Gemini, or OpenRouter)");
                            ui.label("  2. Get an API key from your chosen provider (see 'How to Get API Keys' below)");
                            ui.label("  3. Paste your API key in the corresponding field in Settings");
                            ui.label("  4. Click 'Test API Key' to verify it works");

                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("Step 2: Configure Extraction Fields").strong());
                            ui.label("  1. In the 'Extraction Fields' section, add the data fields you want to extract");
                            ui.label("  2. Examples: Date, Merchant, Total, Tax, Items, Payment Method, etc.");
                            ui.label("  3. Click '+ Add Field' to add each field");

                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("Step 3: Select Your Files").strong());
                            ui.label("  1. Click 'Select Files' to choose receipt images or PDFs");
                            ui.label("  2. You can select multiple files at once");
                            ui.label("  3. Supported formats: JPG, PNG, PDF");

                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("Step 4: Set Output Location").strong());
                            ui.label("  1. In the 'CSV Output' section, click 'Browse'");
                            ui.label("  2. Select where you want to save the extracted data");
                            ui.label("  3. The app will auto-save after each extraction to prevent data loss");

                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("Step 5: Extract Data").strong());
                            ui.label("  1. Click 'Extract Data' to start processing");
                            ui.label("  2. The app will process each file and extract the requested fields");
                            ui.label("  3. Watch the progress bar to see how many files are complete");
                            ui.label("  4. If rate limits are hit, the app will automatically wait and retry");

                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("Step 6: Review and Save").strong());
                            ui.label("  1. After processing, review any incomplete extractions");
                            ui.label("  2. Fill in missing data manually if needed");
                            ui.label("  3. Click 'Save CSV' to save your results");

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // How to Get API Keys
                            ui.label(egui::RichText::new("How to Get API Keys").strong().size(16.0).color(egui::Color32::from_rgb(59, 130, 246)));
                            ui.add_space(10.0);

                            // Groq
                            ui.label(egui::RichText::new("Groq (Recommended for Free Tier)").strong());
                            ui.label("  • Website: https://console.groq.com/keys");
                            ui.label("  • Free tier: 30 requests/minute, very fast");
                            ui.label("  • Steps:");
                            ui.label("    1. Sign up at https://groq.com");
                            ui.label("    2. Go to https://console.groq.com/keys");
                            ui.label("    3. Click 'Create API Key'");
                            ui.label("    4. Copy the key (starts with 'gsk_')");

                            ui.add_space(8.0);

                            // OpenAI
                            ui.label(egui::RichText::new("OpenAI (ChatGPT)").strong());
                            ui.label("  • Website: https://platform.openai.com/api-keys");
                            ui.label("  • Paid service: ~$0.15 per 1000 images");
                            ui.label("  • Steps:");
                            ui.label("    1. Sign up at https://platform.openai.com");
                            ui.label("    2. Add payment method");
                            ui.label("    3. Go to https://platform.openai.com/api-keys");
                            ui.label("    4. Click '+ Create new secret key'");
                            ui.label("    5. Copy the key (starts with 'sk-')");

                            ui.add_space(8.0);

                            // Claude
                            ui.label(egui::RichText::new("Anthropic Claude").strong());
                            ui.label("  • Website: https://console.anthropic.com/settings/keys");
                            ui.label("  • Paid service: ~$0.80 per 1000 images");
                            ui.label("  • Steps:");
                            ui.label("    1. Sign up at https://console.anthropic.com");
                            ui.label("    2. Add payment method");
                            ui.label("    3. Go to https://console.anthropic.com/settings/keys");
                            ui.label("    4. Click 'Create Key'");
                            ui.label("    5. Copy the key (starts with 'sk-ant-')");

                            ui.add_space(8.0);

                            // Gemini
                            ui.label(egui::RichText::new("Google Gemini").strong());
                            ui.label("  • Website: https://aistudio.google.com/app/apikey");
                            ui.label("  • Free tier: 15 requests/minute, 20 requests/day");
                            ui.label("  • Steps:");
                            ui.label("    1. Sign in with Google account");
                            ui.label("    2. Go to https://aistudio.google.com/app/apikey");
                            ui.label("    3. Click 'Create API Key'");
                            ui.label("    4. Copy the key (starts with 'AIza')");
                            ui.label("  • Note: Free tier has daily limits!");

                            ui.add_space(8.0);

                            // OpenRouter
                            ui.label(egui::RichText::new("OpenRouter").strong());
                            ui.label("  • Website: https://openrouter.ai/keys");
                            ui.label("  • Pay-per-use: Access to multiple models");
                            ui.label("  • Steps:");
                            ui.label("    1. Sign up at https://openrouter.ai");
                            ui.label("    2. Add credits to your account");
                            ui.label("    3. Go to https://openrouter.ai/keys");
                            ui.label("    4. Click 'Create Key'");
                            ui.label("    5. Copy the key (starts with 'sk-or-')");

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // Tips and Best Practices
                            ui.label(egui::RichText::new("Tips & Best Practices").strong().size(16.0).color(egui::Color32::from_rgb(59, 130, 246)));
                            ui.add_space(10.0);
                            ui.label("• Start with Groq or Gemini for free testing");
                            ui.label("• Enable 'Auto-save' to prevent data loss during processing");
                            ui.label("• Use 'Stable' models for consistent results");
                            ui.label("• Use 'Latest' models for cutting-edge performance");
                            ui.label("• If you hit rate limits, the app will automatically wait and retry");
                            ui.label("• For best results, use clear, well-lit receipt images");
                            ui.label("• You can process multiple files at once");
                            ui.label("• Review incomplete extractions and fill in missing data manually");

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // Known Limitations Section
                            ui.label(egui::RichText::new("⚠️ Known Limitations").strong().size(16.0).color(egui::Color32::from_rgb(251, 191, 36)));
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("Linux Users:").strong());
                            ui.label("• Drag & drop files from file manager does not work on Linux");
                            ui.label("• This is a known limitation in the windowing library (winit)");
                            ui.label("• Workaround: Use the '+ Add Files' button or press Ctrl+O");

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("Important").strong().color(egui::Color32::from_rgb(251, 191, 36)));
                            ui.label(egui::RichText::new("You must provide your own API keys to use this app. API usage costs are your responsibility.").color(egui::Color32::GRAY).size(11.0));
                        });

                    // Close button - always visible at bottom
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(egui::RichText::new("Close").size(13.0))
                                .min_size(egui::vec2(70.0, 28.0)))
                                .clicked() {
                                self.show_help_modal = false;
                            }
                        });
                    });
                });
        }

        // About Modal Window
        if self.show_about_modal {
            egui::Window::new("About Receipt Data Extractor")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .default_size([400.0, 300.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.heading(egui::RichText::new("🧾 Receipt Data Extractor").size(24.0));
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION"))).color(egui::Color32::GRAY));
                        ui.add_space(20.0);
                    });

                    ui.separator();
                    ui.add_space(10.0);

                    ui.label("A powerful tool for extracting data from receipts and invoices using AI vision models.");

                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Features:").strong());
                    });
                    ui.label("• Support for multiple AI providers (Groq, OpenAI, Claude, Gemini, OpenRouter)");
                    ui.label("• Process images and PDF documents");
                    ui.label("• Customizable extraction fields");
                    ui.label("• Dual extraction mode (summary & detailed items)");
                    ui.label("• Export to CSV format");

                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Built with:").strong());
                    });
                    ui.label("• Rust + egui for the interface");
                    ui.label("• BYOK (Bring Your Own Key) model");

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.vertical_centered(|ui| {
                        if ui.add(egui::Button::new(egui::RichText::new("Close").size(13.0))
                            .min_size(egui::vec2(70.0, 28.0)))
                            .clicked() {
                            self.show_about_modal = false;
                        }
                    });
                });
        }

        // Status Bar at the bottom
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(15, 23, 42))
                .inner_margin(egui::Margin::symmetric(12.0, 6.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Status indicator based on current state
                    let (status_icon, status_text, status_color) = match self.state {
                        AppState::Idle => ("🟢", "Ready", egui::Color32::from_rgb(34, 197, 94)),
                        AppState::Processing => ("🔵", "Processing...", egui::Color32::from_rgb(59, 130, 246)),
                        AppState::ReadyToSave => ("🟡", "Ready to Save", egui::Color32::from_rgb(234, 179, 8)),
                        AppState::Completed => ("✅", "Completed", egui::Color32::from_rgb(34, 197, 94)),
                        AppState::Error => ("🔴", "Error", egui::Color32::from_rgb(239, 68, 68)),
                        AppState::Settings => ("⚙️", "Settings", egui::Color32::GRAY),
                        AppState::Onboarding => ("👋", "Welcome", egui::Color32::from_rgb(59, 130, 246)),
                    };
                    ui.label(egui::RichText::new(status_icon).size(12.0));
                    ui.label(egui::RichText::new(status_text).size(11.0).color(status_color));

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(20.0);

                    // Provider status indicator
                    let api_key = match self.ai_provider {
                        AIProvider::Groq => &self.groq_api_key,
                        AIProvider::ChatGPT => &self.openai_api_key,
                        AIProvider::Claude => &self.claude_api_key,
                        AIProvider::Gemini => &self.gemini_api_key,
                        AIProvider::OpenRouter => &self.openrouter_api_key,
                        AIProvider::LocalLLM => &self.ollama_url,
                    };
                    let has_key = !api_key.is_empty();
                    let key_indicator = if has_key { "🟢" } else { "🔴" };
                    let key_status = if has_key {
                        if self.ai_provider == AIProvider::LocalLLM { "connected" } else { "configured" }
                    } else {
                        if self.ai_provider == AIProvider::LocalLLM { "not configured" } else { "not set" }
                    };
                    ui.label(egui::RichText::new(format!("{} {} ({})", key_indicator, self.ai_provider.name(), key_status))
                        .size(11.0)
                        .color(if has_key { egui::Color32::GRAY } else { egui::Color32::from_rgb(239, 68, 68) }));

                    // Right-aligned: keyboard shortcuts hint
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("Ctrl+O: Open • Ctrl+S: Save • Ctrl+,: Settings • F1: Help")
                            .size(10.0)
                            .color(egui::Color32::from_rgb(100, 116, 139)));
                    });
                });
            });

        // Toast Notification (auto-dismiss after 3 seconds)
        if let Some((message, start_time, toast_type)) = &self.toast_message {
            let elapsed = start_time.elapsed().as_secs_f32();
            if elapsed < 3.0 {
                // Calculate fade out (start fading at 2.5 seconds)
                let alpha = if elapsed > 2.5 { 1.0 - (elapsed - 2.5) * 2.0 } else { 1.0 };
                let alpha_u8 = (alpha * 255.0) as u8;

                let (bg_color, icon) = match toast_type {
                    ToastType::Success => (egui::Color32::from_rgba_unmultiplied(20, 83, 45, alpha_u8), "✅"),
                    ToastType::Error => (egui::Color32::from_rgba_unmultiplied(127, 29, 29, alpha_u8), "❌"),
                    ToastType::Info => (egui::Color32::from_rgba_unmultiplied(30, 58, 138, alpha_u8), "ℹ️"),
                };

                egui::Area::new(egui::Id::new("toast_notification"))
                    .anchor(egui::Align2::CENTER_TOP, [0.0, 60.0])
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        egui::Frame::none()
                            .fill(bg_color)
                            .rounding(egui::Rounding::same(8.0))
                            .inner_margin(egui::Margin::symmetric(20.0, 12.0))
                            .shadow(egui::epaint::Shadow {
                                offset: egui::vec2(0.0, 4.0),
                                blur: 10.0,
                                spread: 0.0,
                                color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, (alpha * 100.0) as u8),
                            })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(icon).size(16.0));
                                    ui.add_space(8.0);
                                    ui.label(egui::RichText::new(message)
                                        .size(14.0)
                                        .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha_u8)));
                                });
                            });
                    });
            } else {
                // Clear the toast after 3 seconds
                self.toast_message = None;
            }
        }

        // Show drag and drop overlay when files are being hovered
        let is_hovering_files = ctx.input(|i| !i.raw.hovered_files.is_empty());
        if is_hovering_files {
            egui::Area::new(egui::Id::new("drop_overlay"))
                .fixed_pos(egui::Pos2::ZERO)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let screen_rect = ctx.screen_rect();
                    ui.allocate_space(screen_rect.size());

                    // Semi-transparent overlay
                    ui.painter().rect_filled(
                        screen_rect,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(30, 41, 59, 230),
                    );

                    // Dashed border effect
                    ui.painter().rect_stroke(
                        screen_rect.shrink(20.0),
                        10.0,
                        egui::Stroke::new(3.0, egui::Color32::from_rgb(59, 130, 246)),
                    );

                    // Center text
                    let center = screen_rect.center();
                    ui.painter().text(
                        center - egui::Vec2::new(0.0, 20.0),
                        egui::Align2::CENTER_CENTER,
                        "📥 Drop files here",
                        egui::FontId::proportional(32.0),
                        egui::Color32::WHITE,
                    );
                    ui.painter().text(
                        center + egui::Vec2::new(0.0, 20.0),
                        egui::Align2::CENTER_CENTER,
                        "Supported: PDF, PNG, JPG, TXT, HTML",
                        egui::FontId::proportional(16.0),
                        egui::Color32::GRAY,
                    );
                });
        }

        ctx.request_repaint();
    }
}

/// Helper function to show a toast notification
impl ReceiptExtractorApp {
    fn show_toast(&mut self, message: &str, toast_type: ToastType) {
        self.toast_message = Some((message.to_string(), std::time::Instant::now(), toast_type));
    }

    /// Trigger background AI prompt enhancement for a custom field
    /// This spawns an async task that calls the AI to generate an optimal extraction prompt
    fn trigger_field_enhancement(&mut self, field_name: String, field_idx: usize, is_item_field: bool) {
        // Get the current API settings
        let api_key = match self.ai_provider {
            AIProvider::Groq => self.groq_api_key.clone(),
            AIProvider::ChatGPT => self.openai_api_key.clone(),
            AIProvider::Claude => self.claude_api_key.clone(),
            AIProvider::Gemini => self.gemini_api_key.clone(),
            AIProvider::OpenRouter => self.openrouter_api_key.clone(),
            AIProvider::LocalLLM => String::new(),
        };

        let is_local = self.ai_provider == AIProvider::LocalLLM;
        let ollama_url_enh = self.ollama_url.clone();
        let local_model_enh = if self.local_model_name.is_empty() { "llava".to_string() } else { self.local_model_name.clone() };

        // If no API key (cloud) or no URL (local) is configured, we can't enhance - just skip
        let can_enhance = if is_local { !ollama_url_enh.is_empty() } else { !api_key.is_empty() };
        if !can_enhance {
            if is_item_field {
                if let Some(field) = self.item_fields.get_mut(field_idx) {
                    field.is_enhancing = false;
                }
            } else {
                if let Some(field) = self.extraction_fields.get_mut(field_idx) {
                    field.is_enhancing = false;
                }
            }
            return;
        }

        let use_latest = self.use_latest_models;

        // Create channel for receiving enhancement result
        let (tx, rx) = std::sync::mpsc::channel();
        self.enhancement_rx = Some(rx);

        // Spawn background task
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut extractor = crate::ai_extractor::AIExtractor::new_with_options(
                    api_key,
                    use_latest
                );
                if is_local {
                    extractor = extractor.with_local_llm(ollama_url_enh, local_model_enh);
                }

                match extractor.enhance_field_prompt(&field_name).await {
                    Ok(prompt) => {
                        let _ = tx.send((field_idx, Some(prompt), is_item_field));
                    }
                    Err(e) => {
                        eprintln!("Failed to enhance prompt for '{}': {}", field_name, e);
                        let _ = tx.send((field_idx, None, is_item_field));
                    }
                }
            });
        });
    }

}

