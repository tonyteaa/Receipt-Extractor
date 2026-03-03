use eframe::egui;
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};
use crate::database::{Database, DatabaseConfig, LicenseKey, DeviceActivation};
use crate::keygen;
use std::fs::File;
use csv::Writer;
use rfd::FileDialog;

#[derive(Debug)]
enum Message {
    DatabaseConnected,
    DatabaseError(String),
    LicensesLoaded(Vec<LicenseKey>),
    KeysGenerated(Vec<String>),
    LicenseAssigned,
    LicenseRevoked,
    DeviceDeactivated,
    NotesSaved,
    EmailResent(String), // Contains success message
    Error(String),
}

#[derive(PartialEq)]
enum Tab {
    Dashboard,
    GenerateKeys,
    ManageLicenses,
    UserManagement,
    Settings,
}

pub struct AdminApp {
    // Database
    db: Option<Database>,
    db_config: DatabaseConfig,
    db_status: String,

    // UI State
    current_tab: Tab,
    message_rx: Option<Receiver<Message>>,

    // Auto-refresh
    last_refresh: Instant,
    refresh_interval: Duration,

    // Dashboard
    licenses: Vec<LicenseKey>,
    filter_tier: String,
    filter_assigned: String,
    filter_active: bool, // Filter for licenses with active devices
    search_email: String,
    search_name: String,
    license_devices: std::collections::HashMap<String, Vec<DeviceActivation>>,

    // Generate Keys
    gen_tier: String,
    gen_count: String,
    generated_keys: Vec<String>,

    // Assign License
    assign_key: String,
    assign_email: String,
    assign_name: String,

    // User Management
    search_query: String,
    search_results: Vec<UserSearchResult>,
    selected_user: Option<UserDetails>,

    // Notes editing
    editing_notes_key: Option<String>,
    editing_notes_text: String,

    // API Configuration
    admin_api_key: String,
    backend_url: String,

    // Status messages
    status_message: String,
    error_message: String,
}

#[derive(Clone)]
struct UserSearchResult {
    email: String,
    name: Option<String>,
    license_count: i32,
}

#[derive(Clone)]
struct UserDetails {
    email: String,
    name: Option<String>,
    licenses: Vec<LicenseKey>,
    activations: Vec<DeviceActivation>,
}

impl AdminApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            db: None,
            db_config: DatabaseConfig::default(),
            db_status: "Not connected".to_string(),
            current_tab: Tab::Dashboard,
            message_rx: None,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(5), // Refresh every 5 seconds
            licenses: Vec::new(),
            filter_tier: "ALL".to_string(),
            filter_assigned: "ALL".to_string(),
            filter_active: false,
            search_email: String::new(),
            search_name: String::new(),
            license_devices: std::collections::HashMap::new(),
            gen_tier: "PRO".to_string(),
            gen_count: "10".to_string(),
            generated_keys: Vec::new(),
            assign_key: String::new(),
            assign_email: String::new(),
            assign_name: String::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            selected_user: None,
            editing_notes_key: None,
            editing_notes_text: String::new(),
            admin_api_key: std::env::var("ADMIN_API_KEY")
                .unwrap_or_else(|_| "b57055be5e63633c9114ba2b03832faa4ea8ef2f9186b77855f49a8231a1b610".to_string()),
            backend_url: std::env::var("RECEIPT_API_URL")
                .unwrap_or_else(|_| "https://clever-vision-production.up.railway.app".to_string()),
            status_message: String::new(),
            error_message: String::new(),
        };
        
        // Try to connect to database on startup
        app.connect_database();
        
        app
    }
    
    fn connect_database(&mut self) {
        let config = self.db_config.clone();
        let (tx, rx) = channel();
        self.message_rx = Some(rx);
        
        std::thread::spawn(move || {
            match Database::new(config) {
                Ok(db) => {
                    if db.test_connection().is_ok() {
                        let _ = tx.send(Message::DatabaseConnected);
                        
                        // Load licenses
                        if let Ok(licenses) = db.get_all_licenses() {
                            let _ = tx.send(Message::LicensesLoaded(licenses));
                        }
                    } else {
                        let _ = tx.send(Message::DatabaseError("Connection test failed".to_string()));
                    }
                }
                Err(e) => {
                    let _ = tx.send(Message::DatabaseError(e.to_string()));
                }
            }
        });
    }

    fn refresh_licenses(&mut self) {
        if let Some(db) = &self.db {
            let db_clone = db.clone();
            let (tx, rx) = channel();
            self.message_rx = Some(rx);

            std::thread::spawn(move || {
                if let Ok(licenses) = db_clone.get_all_licenses() {
                    let _ = tx.send(Message::LicensesLoaded(licenses));
                }
            });
        }
    }

    fn load_all_device_activations(&mut self) {
        if let Some(db) = &self.db {
            // Load devices for all licenses that have activations
            for license in &self.licenses {
                if license.activation_count > 0 {
                    let db_clone = db.clone();
                    let key = license.license_key.clone();

                    if let Ok(devices) = db_clone.get_license_activations(&key) {
                        self.license_devices.insert(key, devices);
                    }
                }
            }
        }
    }

    fn export_to_csv(&mut self, licenses: &[LicenseKey]) {
        // Generate default filename with timestamp
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let default_filename = format!("licenses_export_{}.csv", timestamp);

        // Show save file dialog
        let file_path = FileDialog::new()
            .add_filter("CSV Files", &["csv"])
            .add_filter("All Files", &["*"])
            .set_file_name(&default_filename)
            .set_title("Export Licenses to CSV")
            .save_file();

        // If user cancelled the dialog, return early
        let file_path = match file_path {
            Some(path) => path,
            None => {
                self.status_message = "Export cancelled.".to_string();
                return;
            }
        };

        match File::create(&file_path) {
            Ok(file) => {
                let mut wtr = Writer::from_writer(file);

                // Write header
                if let Err(e) = wtr.write_record(&[
                    "License Key", "Tier", "Status", "Customer Email", "Customer Name",
                    "Created At", "Assigned At", "Revoked At", "Active Devices", "Max Devices"
                ]) {
                    self.error_message = format!("Failed to write CSV header: {}", e);
                    return;
                }

                // Write data
                for license in licenses {
                    let status = if license.revoked_at.is_some() {
                        "Revoked"
                    } else if license.is_used {
                        "Assigned"
                    } else {
                        "Available"
                    };

                    if let Err(e) = wtr.write_record(&[
                        &license.license_key,
                        &license.tier.to_uppercase(),
                        status,
                        license.customer_email.as_deref().unwrap_or(""),
                        license.customer_name.as_deref().unwrap_or(""),
                        &license.created_at,
                        license.assigned_at.as_deref().unwrap_or(""),
                        license.revoked_at.as_deref().unwrap_or(""),
                        &license.activation_count.to_string(),
                        &license.max_activations.to_string(),
                    ]) {
                        self.error_message = format!("Failed to write CSV row: {}", e);
                        return;
                    }
                }

                if let Err(e) = wtr.flush() {
                    self.error_message = format!("Failed to save CSV: {}", e);
                    return;
                }

                self.status_message = format!("✅ Exported {} licenses to {}", licenses.len(), file_path.display());
                self.error_message.clear();
            }
            Err(e) => {
                self.error_message = format!("Failed to create CSV file: {}", e);
            }
        }
    }

    fn generate_keys(&mut self) {
        let tier = self.gen_tier.clone();
        let count: usize = self.gen_count.parse().unwrap_or(10);
        
        if let Some(db) = &self.db {
            let db_clone = db.clone();
            let (tx, rx) = channel();
            self.message_rx = Some(rx);
            
            std::thread::spawn(move || {
                let mut keys = Vec::new();
                
                for _ in 0..count {
                    let key = keygen::generate_license_key(&tier);
                    
                    if let Err(e) = db_clone.insert_license(&key, &tier) {
                        let _ = tx.send(Message::Error(format!("Failed to insert key: {}", e)));
                        return;
                    }
                    
                    keys.push(key);
                }
                
                let _ = tx.send(Message::KeysGenerated(keys));
                
                // Reload licenses
                if let Ok(licenses) = db_clone.get_all_licenses() {
                    let _ = tx.send(Message::LicensesLoaded(licenses));
                }
            });
        }
    }
    
    fn assign_license(&mut self) {
        if let Some(db) = &self.db {
            let key = self.assign_key.clone();
            let email = self.assign_email.clone();
            let name = self.assign_name.clone();
            
            let db_clone = db.clone();
            let (tx, rx) = channel();
            self.message_rx = Some(rx);
            
            std::thread::spawn(move || {
                match db_clone.assign_license(&key, &email, &name) {
                    Ok(_) => {
                        let _ = tx.send(Message::LicenseAssigned);
                        
                        // Reload licenses
                        if let Ok(licenses) = db_clone.get_all_licenses() {
                            let _ = tx.send(Message::LicensesLoaded(licenses));
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Message::Error(format!("Failed to assign license: {}", e)));
                    }
                }
            });
        }
    }

    fn revoke_license(&mut self, license_key: &str) {
        if let Some(db) = &self.db {
            let key = license_key.to_string();
            let db_clone = db.clone();
            let (tx, rx) = channel();
            self.message_rx = Some(rx);

            std::thread::spawn(move || {
                match db_clone.revoke_license(&key) {
                    Ok(_) => {
                        let _ = tx.send(Message::LicenseRevoked);

                        // Reload licenses
                        if let Ok(licenses) = db_clone.get_all_licenses() {
                            let _ = tx.send(Message::LicensesLoaded(licenses));
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Message::Error(format!("Failed to revoke license: {}", e)));
                    }
                }
            });
        }
    }

    fn deactivate_device(&mut self, activation_id: i32) {
        if let Some(db) = &self.db {
            let db_clone = db.clone();
            let (tx, rx) = channel();
            self.message_rx = Some(rx);

            std::thread::spawn(move || {
                match db_clone.deactivate_device(activation_id) {
                    Ok(_) => {
                        let _ = tx.send(Message::DeviceDeactivated);

                        // Reload licenses
                        if let Ok(licenses) = db_clone.get_all_licenses() {
                            let _ = tx.send(Message::LicensesLoaded(licenses));
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Message::Error(format!("Failed to deactivate device: {}", e)));
                    }
                }
            });
        }
    }

    fn save_license_notes(&mut self, license_key: &str, notes: &str) {
        if let Some(db) = &self.db {
            let key = license_key.to_string();
            let notes = notes.to_string();
            let db_clone = db.clone();
            let (tx, rx) = channel();
            self.message_rx = Some(rx);

            std::thread::spawn(move || {
                match db_clone.update_license_notes(&key, &notes) {
                    Ok(_) => {
                        let _ = tx.send(Message::NotesSaved);

                        // Reload licenses
                        if let Ok(licenses) = db_clone.get_all_licenses() {
                            let _ = tx.send(Message::LicensesLoaded(licenses));
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Message::Error(format!("Failed to save notes: {}", e)));
                    }
                }
            });
        }
    }

    fn resend_license_email(&mut self, license_key: &str, email: &str, name: &str, tier: &str) {
        let key = license_key.to_string();
        let email = email.to_string();
        let name = name.to_string();
        let tier = tier.to_string();
        let backend_url = self.backend_url.clone();
        let api_key = self.admin_api_key.clone();
        let (tx, rx) = channel();
        self.message_rx = Some(rx);

        std::thread::spawn(move || {
            // Call the backend API to resend the email
            let client = reqwest::blocking::Client::new();
            let result = client
                .post(format!("{}/api/admin/resend-email", backend_url))
                .header("X-Admin-API-Key", api_key)
                .json(&serde_json::json!({
                    "license_key": key,
                    "email": email,
                    "name": name,
                    "tier": tier
                }))
                .send();

            match result {
                Ok(response) => {
                    if response.status().is_success() {
                        let _ = tx.send(Message::EmailResent(format!("Email sent to {}", email)));
                    } else {
                        let _ = tx.send(Message::Error(format!("Failed to send email: {}", response.status())));
                    }
                }
                Err(e) => {
                    let _ = tx.send(Message::Error(format!("Failed to send email: {}", e)));
                }
            }
        });
    }

    fn search_users(&mut self) {
        if let Some(db) = &self.db {
            let query = self.search_query.clone();
            match db.search_users(&query) {
                Ok(results) => {
                    self.search_results = results.into_iter()
                        .map(|(email, name, count)| UserSearchResult {
                            email,
                            name,
                            license_count: count,
                        })
                        .collect();
                    self.status_message = format!("Found {} user(s)", self.search_results.len());
                }
                Err(e) => {
                    self.error_message = format!("Search failed: {}", e);
                }
            }
        }
    }

    fn load_user_details(&mut self, email: &str) {
        if let Some(db) = &self.db {
            match (db.get_user_licenses(email), db.get_user_activations(email)) {
                (Ok(licenses), Ok(activations)) => {
                    let name = licenses.first()
                        .and_then(|l| l.customer_name.clone());

                    self.selected_user = Some(UserDetails {
                        email: email.to_string(),
                        name,
                        licenses,
                        activations,
                    });
                }
                (Err(e), _) | (_, Err(e)) => {
                    self.error_message = format!("Failed to load user details: {}", e);
                }
            }
        }
    }
}

impl eframe::App for AdminApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-refresh licenses every 5 seconds
        if self.db.is_some() && self.last_refresh.elapsed() >= self.refresh_interval {
            self.refresh_licenses();
            self.last_refresh = Instant::now();
        }

        // Request repaint for continuous updates
        ctx.request_repaint_after(Duration::from_secs(1));

        // Handle messages
        let mut should_clear_rx = false;
        let mut should_load_devices = false;
        if let Some(rx) = &self.message_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    Message::DatabaseConnected => {
                        self.db = Some(Database::new(self.db_config.clone()).unwrap());
                        self.db_status = "✅ Connected".to_string();
                        self.status_message = "Database connected successfully!".to_string();
                        should_clear_rx = true;
                    }
                    Message::DatabaseError(err) => {
                        self.db_status = format!("❌ Error: {}", err);
                        self.error_message = err;
                        should_clear_rx = true;
                    }
                    Message::LicensesLoaded(licenses) => {
                        self.licenses = licenses;
                        self.last_refresh = Instant::now(); // Reset timer when licenses loaded
                        should_load_devices = true; // Flag to load devices after we release the borrow
                    }
                    Message::KeysGenerated(keys) => {
                        self.generated_keys = keys;
                        self.status_message = format!("Generated {} license keys!", self.generated_keys.len());
                    }
                    Message::LicenseAssigned => {
                        self.status_message = "License assigned successfully!".to_string();
                        self.assign_key.clear();
                        self.assign_email.clear();
                        self.assign_name.clear();
                    }
                    Message::LicenseRevoked => {
                        self.status_message = "License revoked successfully!".to_string();
                    }
                    Message::DeviceDeactivated => {
                        self.status_message = "Device deactivated successfully!".to_string();
                    }
                    Message::NotesSaved => {
                        self.status_message = "Notes saved successfully!".to_string();
                        self.editing_notes_key = None;
                        self.editing_notes_text.clear();
                    }
                    Message::EmailResent(msg) => {
                        self.status_message = msg;
                    }
                    Message::Error(err) => {
                        self.error_message = err;
                    }
                }
            }
        }
        if should_clear_rx {
            self.message_rx = None;
        }

        // Load device activations after releasing the borrow
        if should_load_devices {
            self.load_all_device_activations();
        }

        // Top panel - Header
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("📊 Receipt Extractor - Admin Tool");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(&self.db_status);
                });
            });
        });

        // Bottom panel - Status
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if !self.status_message.is_empty() {
                    ui.colored_label(egui::Color32::GREEN, &self.status_message);
                }
                if !self.error_message.is_empty() {
                    ui.colored_label(egui::Color32::RED, &self.error_message);
                }
            });
        });

        // Side panel - Navigation
        egui::SidePanel::left("nav").min_width(150.0).show(ctx, |ui| {
            ui.heading("Navigation");
            ui.separator();

            if ui.selectable_label(self.current_tab == Tab::Dashboard, "📊 Dashboard").clicked() {
                self.current_tab = Tab::Dashboard;
            }
            if ui.selectable_label(self.current_tab == Tab::GenerateKeys, "🔑 Generate Keys").clicked() {
                self.current_tab = Tab::GenerateKeys;
            }
            if ui.selectable_label(self.current_tab == Tab::ManageLicenses, "📝 Manage Licenses").clicked() {
                self.current_tab = Tab::ManageLicenses;
            }
            if ui.selectable_label(self.current_tab == Tab::UserManagement, "👥 User Management").clicked() {
                self.current_tab = Tab::UserManagement;
            }
            if ui.selectable_label(self.current_tab == Tab::Settings, "⚙️ Settings").clicked() {
                self.current_tab = Tab::Settings;
            }
        });

        // Central panel - Content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Dashboard => self.show_dashboard(ui),
                Tab::GenerateKeys => self.show_generate_keys(ui),
                Tab::ManageLicenses => self.show_manage_licenses(ui),
                Tab::UserManagement => self.show_user_management(ui),
                Tab::Settings => self.show_settings(ui),
            }
        });
    }
}

impl AdminApp {
    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 Dashboard");
        ui.separator();

        // Statistics
        let total_licenses = self.licenses.len();
        let used_licenses = self.licenses.iter().filter(|l| l.is_used).count();
        let available_licenses = total_licenses - used_licenses;

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading(total_licenses.to_string());
                    ui.label("Total Licenses");
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading(used_licenses.to_string());
                    ui.label("Used Licenses");
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading(available_licenses.to_string());
                    ui.label("Available");
                });
            });
        });

        ui.add_space(20.0);

        // Recent licenses with refresh button
        ui.horizontal(|ui| {
            ui.heading("Recent Licenses");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Refresh").clicked() {
                    self.refresh_licenses();
                }
                let seconds_since_refresh = self.last_refresh.elapsed().as_secs();
                ui.label(format!("Updated {}s ago", seconds_since_refresh));
            });
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for license in self.licenses.iter().take(10) {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(&license.license_key);
                            ui.label(format!("[{}]", license.tier.to_uppercase()));

                            if license.is_used {
                                ui.colored_label(egui::Color32::GREEN, "✓ Used");
                                if let Some(email) = &license.customer_email {
                                    ui.label(email);
                                }
                            } else {
                                ui.colored_label(egui::Color32::GRAY, "○ Available");
                            }
                        });

                        // Show activation status
                        if license.activation_count > 0 {
                            ui.horizontal(|ui| {
                                ui.label(format!("💻 Devices: {}/{}", license.activation_count, license.max_activations));

                                // Color code based on usage
                                if license.activation_count >= license.max_activations {
                                    ui.colored_label(egui::Color32::RED, "⚠ Full");
                                } else if license.activation_count > 0 {
                                    ui.colored_label(egui::Color32::YELLOW, "Active");
                                }
                            });
                        }
                    });
                });
            }
        });
    }

    fn show_generate_keys(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔑 Generate License Keys");
        ui.separator();

        // All licenses are Pro tier now
        self.gen_tier = "PRO".to_string();

        ui.horizontal(|ui| {
            ui.label("Count:");
            ui.text_edit_singleline(&mut self.gen_count);
        });

        ui.add_space(10.0);

        if ui.button("🎲 Generate Keys").clicked() {
            self.generate_keys();
        }

        ui.add_space(20.0);

        if !self.generated_keys.is_empty() {
            ui.heading("Generated Keys:");
            ui.separator();

            egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                for key in &self.generated_keys {
                    ui.horizontal(|ui| {
                        ui.label(key);
                        if ui.button("📋 Copy").clicked() {
                            ui.output_mut(|o| o.copied_text = key.clone());
                        }
                    });
                }
            });
        }
    }

    fn show_manage_licenses(&mut self, ui: &mut egui::Ui) {
        ui.heading("📝 Manage Licenses");
        ui.separator();

        // Assign license section
        ui.group(|ui| {
            ui.heading("Assign License to Customer");

            ui.horizontal(|ui| {
                ui.label("License Key:");
                ui.text_edit_singleline(&mut self.assign_key);
            });

            ui.horizontal(|ui| {
                ui.label("Customer Email:");
                ui.text_edit_singleline(&mut self.assign_email);
            });

            ui.horizontal(|ui| {
                ui.label("Customer Name:");
                ui.text_edit_singleline(&mut self.assign_name);
            });

            if ui.button("✅ Assign License").clicked() {
                self.assign_license();
            }
        });

        ui.add_space(20.0);

        // Filters
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.filter_active, "🖥️ Active devices only");
        });

        ui.horizontal(|ui| {
            ui.label("Status:");
            ui.radio_value(&mut self.filter_assigned, "ALL".to_string(), "All");
            ui.radio_value(&mut self.filter_assigned, "ASSIGNED".to_string(), "Assigned");
            ui.radio_value(&mut self.filter_assigned, "UNASSIGNED".to_string(), "Unassigned");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Refresh").clicked() {
                    self.refresh_licenses();
                }
                let seconds_since_refresh = self.last_refresh.elapsed().as_secs();
                ui.label(format!("Updated {}s ago", seconds_since_refresh));
            });
        });

        // Search filters
        ui.horizontal(|ui| {
            ui.label("🔍 Search:");
            ui.label("Email:");
            ui.text_edit_singleline(&mut self.search_email);
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.search_name);

            if ui.button("Clear").clicked() {
                self.search_email.clear();
                self.search_name.clear();
            }
        });

        ui.separator();

        // License list
        let mut license_to_revoke: Option<String> = None;
        let mut device_to_deactivate: Option<i32> = None;
        let mut email_to_resend: Option<(String, String, String, String)> = None; // (license_key, email, name, tier)
        let mut notes_to_edit: Option<(String, String)> = None; // (license_key, current_notes)
        let mut notes_to_save: Option<(String, String)> = None; // (license_key, new_notes)
        let mut should_export = false;

        // Filter licenses based on search criteria
        let filtered_licenses: Vec<LicenseKey> = self.licenses.iter()
            .filter(|license| {
                // Apply tier filter
                if self.filter_tier != "ALL" && license.tier != self.filter_tier {
                    return false;
                }

                // Apply assigned filter
                if self.filter_assigned == "ASSIGNED" && !license.is_used {
                    return false;
                }
                if self.filter_assigned == "UNASSIGNED" && license.is_used {
                    return false;
                }

                // Apply active devices filter
                if self.filter_active && license.activation_count == 0 {
                    return false;
                }

                // Apply email search
                if !self.search_email.is_empty() {
                    if let Some(email) = &license.customer_email {
                        if !email.to_lowercase().contains(&self.search_email.to_lowercase()) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                // Apply name search
                if !self.search_name.is_empty() {
                    if let Some(name) = &license.customer_name {
                        if !name.to_lowercase().contains(&self.search_name.to_lowercase()) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Export button
        ui.horizontal(|ui| {
            ui.label(format!("Showing {} licenses", filtered_licenses.len()));

            if ui.button("📥 Export to CSV").clicked() {
                should_export = true;
            }
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for license in &filtered_licenses {

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(format!("Key: {}", license.license_key));
                                ui.label(format!("Tier: {}", license.tier.to_uppercase()));

                                if license.is_used {
                                    if let Some(email) = &license.customer_email {
                                        ui.label(format!("Customer: {}", email));
                                    }
                                    if let Some(name) = &license.customer_name {
                                        ui.label(format!("Name: {}", name));
                                    }
                                    if let Some(assigned_at) = &license.assigned_at {
                                        ui.label(format!("Assigned: {}", assigned_at));
                                    }
                                }

                                // Show revoked status
                                if let Some(revoked_at) = &license.revoked_at {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(255, 100, 100),
                                        format!("❌ REVOKED: {}", revoked_at)
                                    );
                                }
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if license.revoked_at.is_some() {
                                    ui.label("🚫 Revoked");
                                } else if license.is_used {
                                    // Assigned license - can revoke
                                    if ui.button("🔓 Revoke").clicked() {
                                        license_to_revoke = Some(license.license_key.clone());
                                    }
                                } else if license.activation_count > 0 {
                                    // Test license with activations - can clear activations
                                    if ui.button("🗑️ Clear Activations").clicked() {
                                        license_to_revoke = Some(license.license_key.clone());
                                    }
                                }
                            });
                        });

                        // Show activation status and devices
                        if license.activation_count > 0 {
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label(format!("💻 Active Devices: {}/{}", license.activation_count, license.max_activations));

                                // Color code based on usage
                                if license.activation_count >= license.max_activations {
                                    ui.colored_label(egui::Color32::RED, "⚠ Maximum reached");
                                } else if license.activation_count > 0 {
                                    ui.colored_label(egui::Color32::YELLOW, "✓ Active");
                                }
                            });

                            // Always show device list
                            if let Some(devices) = self.license_devices.get(&license.license_key) {
                                ui.add_space(5.0);
                                for device in devices {
                                    ui.indent(format!("device_{}", device.id), |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label("🖥️");
                                            ui.vertical(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(format!("Device: {}", device.device_name));
                                                    if ui.small_button("🗑️").on_hover_text("Deactivate this device").clicked() {
                                                        device_to_deactivate = Some(device.id);
                                                    }
                                                });
                                                if let Some(ip) = &device.ip_address {
                                                    ui.label(format!("IP: {}", ip));
                                                }
                                                ui.label(format!("Last Seen: {}", device.last_seen));

                                                if device.is_active {
                                                    ui.colored_label(egui::Color32::GREEN, "✓ Active");
                                                } else {
                                                    ui.colored_label(egui::Color32::GRAY, "○ Inactive");
                                                }
                                            });
                                        });
                                    });
                                    ui.add_space(3.0);
                                }
                            }
                        }

                        // Notes section
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("📝 Notes:");
                            if self.editing_notes_key.as_ref() == Some(&license.license_key) {
                                // Editing mode - show text input
                                ui.add(egui::TextEdit::singleline(&mut self.editing_notes_text).desired_width(300.0));
                                if ui.small_button("💾 Save").clicked() {
                                    notes_to_save = Some((license.license_key.clone(), self.editing_notes_text.clone()));
                                }
                                if ui.small_button("❌ Cancel").clicked() {
                                    self.editing_notes_key = None;
                                    self.editing_notes_text.clear();
                                }
                            } else {
                                // Display mode
                                if let Some(notes) = &license.notes {
                                    if !notes.is_empty() {
                                        ui.label(notes);
                                    } else {
                                        ui.colored_label(egui::Color32::GRAY, "(no notes)");
                                    }
                                } else {
                                    ui.colored_label(egui::Color32::GRAY, "(no notes)");
                                }
                                if ui.small_button("✏️ Edit").clicked() {
                                    notes_to_edit = Some((license.license_key.clone(), license.notes.clone().unwrap_or_default()));
                                }
                            }
                        });

                        // Resend email button (only for assigned licenses)
                        if license.is_used && license.customer_email.is_some() {
                            ui.horizontal(|ui| {
                                if ui.button("📧 Resend License Email").clicked() {
                                    email_to_resend = Some((
                                        license.license_key.clone(),
                                        license.customer_email.clone().unwrap_or_default(),
                                        license.customer_name.clone().unwrap_or_default(),
                                        license.tier.clone(),
                                    ));
                                }
                            });
                        }
                    });
                });
            }
        });

        // Handle actions after the loop
        if let Some(key) = license_to_revoke {
            self.revoke_license(&key);
        }

        if let Some(device_id) = device_to_deactivate {
            self.deactivate_device(device_id);
        }

        if let Some((key, notes)) = notes_to_edit {
            self.editing_notes_key = Some(key);
            self.editing_notes_text = notes;
        }

        if let Some((key, notes)) = notes_to_save {
            self.save_license_notes(&key, &notes);
        }

        if let Some((key, email, name, tier)) = email_to_resend {
            self.resend_license_email(&key, &email, &name, &tier);
        }

        if should_export {
            self.export_to_csv(&filtered_licenses);
        }
    }

    fn show_user_management(&mut self, ui: &mut egui::Ui) {
        ui.heading("👥 User Management");
        ui.separator();

        if self.db.is_none() {
            ui.colored_label(egui::Color32::RED, "⚠️ Database not connected");
            return;
        }

        // Search bar
        ui.horizontal(|ui| {
            ui.label("🔍 Search users:");
            let response = ui.text_edit_singleline(&mut self.search_query);

            if ui.button("Search").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                self.search_users();
            }
        });

        ui.add_space(10.0);

        // Two-column layout
        ui.columns(2, |columns| {
            // Left column - Search results
            columns[0].heading("Search Results");
            columns[0].separator();

            egui::ScrollArea::vertical().max_height(500.0).show(&mut columns[0], |ui| {
                if self.search_results.is_empty() {
                    ui.label("No users found. Try searching by email or name.");
                } else {
                    for user in &self.search_results.clone() {
                        let is_selected = self.selected_user.as_ref()
                            .map(|u| u.email == user.email)
                            .unwrap_or(false);

                        if ui.selectable_label(is_selected, format!("📧 {}\n   {} license(s)",
                            user.email, user.license_count)).clicked() {
                            self.load_user_details(&user.email);
                        }
                    }
                }
            });

            // Right column - User details
            columns[1].heading("User Details");
            columns[1].separator();

            if let Some(user) = &self.selected_user.clone() {
                egui::ScrollArea::vertical().max_height(500.0).show(&mut columns[1], |ui| {
                    ui.group(|ui| {
                        ui.label(format!("📧 Email: {}", user.email));
                        if let Some(name) = &user.name {
                            ui.label(format!("👤 Name: {}", name));
                        }
                    });

                    ui.add_space(10.0);

                    // Licenses section
                    ui.heading("🔑 Licenses");
                    ui.separator();

                    for license in &user.licenses {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("Key: {}", license.license_key));
                                ui.label(format!("| Tier: {}", license.tier));
                            });
                            ui.label(format!("Created: {}", license.created_at));
                            if let Some(assigned) = &license.assigned_at {
                                ui.label(format!("Assigned: {}", assigned));
                            }

                            // Show activations for this license
                            let license_activations: Vec<_> = user.activations.iter()
                                .filter(|a| a.license_key == license.license_key)
                                .collect();

                            if !license_activations.is_empty() {
                                ui.label(format!("💻 Devices: {}", license_activations.len()));
                            }
                        });
                        ui.add_space(5.0);
                    }

                    ui.add_space(10.0);

                    // Devices section
                    ui.heading("💻 Device Activations");
                    ui.separator();

                    if user.activations.is_empty() {
                        ui.label("No device activations");
                    } else {
                        for activation in &user.activations {
                            ui.group(|ui| {
                                let status_color = if activation.is_active {
                                    egui::Color32::GREEN
                                } else {
                                    egui::Color32::RED
                                };
                                let status_text = if activation.is_active { "✅ Active" } else { "❌ Inactive" };

                                ui.colored_label(status_color, status_text);
                                ui.label(format!("Device: {}", activation.device_name));
                                ui.label(format!("Fingerprint: {}...",
                                    activation.device_fingerprint.chars().take(16).collect::<String>()));
                                ui.label(format!("Activated: {}", activation.activated_at));
                                ui.label(format!("Last seen: {}", activation.last_seen));

                                if activation.is_active {
                                    if ui.button("🚫 Deactivate Device").clicked() {
                                        self.deactivate_device(activation.id);
                                    }
                                }
                            });
                            ui.add_space(5.0);
                        }
                    }
                });
            } else {
                columns[1].label("Select a user from the search results to view details");
            }
        });
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ Settings");
        ui.separator();

        ui.label("Database Configuration:");

        ui.horizontal(|ui| {
            ui.label("Host:");
            ui.text_edit_singleline(&mut self.db_config.host);
        });

        ui.horizontal(|ui| {
            ui.label("Port:");
            let mut port_str = self.db_config.port.to_string();
            if ui.text_edit_singleline(&mut port_str).changed() {
                if let Ok(port) = port_str.parse() {
                    self.db_config.port = port;
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("User:");
            ui.text_edit_singleline(&mut self.db_config.user);
        });

        ui.horizontal(|ui| {
            ui.label("Password:");
            ui.add(egui::TextEdit::singleline(&mut self.db_config.password).password(true));
        });

        ui.horizontal(|ui| {
            ui.label("Database:");
            ui.text_edit_singleline(&mut self.db_config.database);
        });

        ui.add_space(10.0);

        if ui.button("🔄 Reconnect").clicked() {
            self.connect_database();
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        ui.label("API Configuration:");

        ui.horizontal(|ui| {
            ui.label("Backend URL:");
            ui.text_edit_singleline(&mut self.backend_url);
        });

        ui.horizontal(|ui| {
            ui.label("Admin API Key:");
            ui.add(egui::TextEdit::singleline(&mut self.admin_api_key).password(true));
        });
    }
}

