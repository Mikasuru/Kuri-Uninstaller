mod theme;

use iced::{
    widget::{button, checkbox, column, container, scrollable, text, Space},
    executor, Application, Command, Element, Length, Settings, Size,
};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;
use is_elevated;

// Entry point
pub fn main() -> iced::Result {
    if !is_elevated::is_elevated() {
        eprintln!("\n[ERROR] Administrator Privileges Required");
        eprintln!("This application needs to be run as an administrator to delete system-wide files and registry keys.");
        eprintln!("Please right-click the executable and select 'Run as administrator'.\n");
        // Block for a moment so the user can read the message in the console
        std::thread::sleep(std::time::Duration::from_secs(5));
        // We must return a specific iced::Error variant.
        return Err(iced::Error::WindowCreationFailed(Box::new(
            std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Administrator privileges required.",
            ),
        )));
    }

    KuriUninstaller::run(Settings {
        window: iced::window::Settings {
            size: Size::new(800.0, 600.0),
            ..Default::default()
        },
        ..Default::default()
    })
}

// --- Data Structures ---

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProgramInfo {
    name: String,
    version: String,
    install_location: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FoundItem {
    File(PathBuf),
    Directory(PathBuf),
    RegistryKey(String),
}

impl std::fmt::Display for FoundItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FoundItem::File(path) => write!(f, "[File] {}", path.display()),
            FoundItem::Directory(path) => write!(f, "[Folder] {}", path.display()),
            FoundItem::RegistryKey(key) => write!(f, "[Registry] {}", key),
        }
    }
}

#[derive(Debug, Default)]
enum ViewState {
    #[default]
    ProgramList,
    Scanning,
    ScanResults,
    ConfirmingDelete,
    Deleting,
}

struct KuriUninstaller {
    programs: Vec<ProgramInfo>,
    selected_program: Option<ProgramInfo>,
    scan_results: Vec<(FoundItem, bool)>,
    view_state: ViewState,
    error_message: Option<String>,
    backup_registry: bool,
}

// --- Messages for UI interaction ---

#[derive(Debug, Clone)]
enum Message {
    LoadPrograms(Result<Vec<ProgramInfo>, String>),
    ProgramSelected(ProgramInfo),
    ScanButtonPressed,
    ScanCompleted(Result<Vec<FoundItem>, String>),
    ResultChecked(usize, bool),
    SelectAll,
    DeselectAll,
    DeleteSelectedButtonPressed,
    ConfirmDelete,
    CancelDelete,
    BackupCheckboxToggled(bool),
    DeleteCompleted(Result<(), String>),
    BackButtonPressed,
    DismissError,
}

// --- Application Logic ---

impl Application for KuriUninstaller {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = theme::Fluent;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            KuriUninstaller {
                programs: vec![],
                selected_program: None,
                scan_results: vec![],
                view_state: ViewState::default(),
                error_message: None,
                backup_registry: true,
            },
            Command::perform(load_installed_programs(), Message::LoadPrograms),
        )
    }

    fn title(&self) -> String {
        String::from("Kuri Uninstaller")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        if !matches!(message, Message::DismissError) {
            self.error_message = None;
        }

        match message {
            Message::LoadPrograms(Ok(programs)) => self.programs = programs,
            Message::LoadPrograms(Err(e)) => self.error_message = Some(format!("Failed to load programs: {}", e)),
            Message::ProgramSelected(program) => self.selected_program = Some(program),
            Message::ScanButtonPressed => {
                if let Some(program) = self.selected_program.clone() {
                    self.view_state = ViewState::Scanning;
                    return Command::perform(scan_for_leftovers(program), Message::ScanCompleted);
                }
            }
            Message::ScanCompleted(Ok(results)) => {
                self.scan_results = results.into_iter().map(|item| (item, true)).collect();
                self.view_state = ViewState::ScanResults;
            }
            Message::ScanCompleted(Err(e)) => {
                self.error_message = Some(format!("Error during scan: {}", e));
                self.view_state = ViewState::ProgramList;
            }
            Message::ResultChecked(index, is_checked) => {
                if let Some(item) = self.scan_results.get_mut(index) {
                    item.1 = is_checked;
                }
            }
            Message::SelectAll => self.scan_results.iter_mut().for_each(|(_, c)| *c = true),
            Message::DeselectAll => self.scan_results.iter_mut().for_each(|(_, c)| *c = false),
            Message::DeleteSelectedButtonPressed => self.view_state = ViewState::ConfirmingDelete,
            Message::BackupCheckboxToggled(is_checked) => self.backup_registry = is_checked,
            Message::ConfirmDelete => {
                self.view_state = ViewState::Deleting;
                let items_to_delete: Vec<FoundItem> = self
                    .scan_results
                    .iter()
                    .filter(|(_, c)| *c)
                    .map(|(item, _)| item.clone())
                    .collect();
                return Command::perform(
                    delete_items(items_to_delete, self.backup_registry),
                    Message::DeleteCompleted,
                );
            }
            Message::CancelDelete => self.view_state = ViewState::ScanResults,
            Message::BackButtonPressed => {
                self.view_state = ViewState::ProgramList;
                self.selected_program = None;
                self.scan_results = vec![];
            }
            Message::DeleteCompleted(Ok(())) => {
                self.view_state = ViewState::ProgramList;
                self.selected_program = None;
                self.scan_results = vec![];
                return Command::perform(load_installed_programs(), Message::LoadPrograms);
            }
            Message::DeleteCompleted(Err(e)) => {
                self.error_message = Some(format!("An error occurred: {}", e));
                self.view_state = ViewState::ScanResults;
            }
            Message::DismissError => self.error_message = None,
        }
        Command::none()
    }

    fn view(&self) -> Element<Message, Self::Theme> {
        let main_content = match self.view_state {
            ViewState::ProgramList => self.view_program_list(),
            ViewState::Scanning => self.view_loading("Scanning..."),
            ViewState::ScanResults => self.view_scan_results(),
            ViewState::ConfirmingDelete => self.view_confirm_delete(),
            ViewState::Deleting => self.view_loading("Deleting items..."),
        };

        let content = if let Some(error) = &self.error_message {
            let error_content = column![
                text("Error").size(24).style(theme::Text::Error),
                text(error).size(16).style(theme::Text::Error),
                button("Dismiss").on_press(Message::DismissError).padding(10),
            ]
            .spacing(10)
            .padding(20)
            .align_items(iced::Alignment::Center);

            container(container(error_content).style(theme::Container::Error))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into()
        } else {
            main_content
        };
        
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .padding(20)
            .into()
    }
}

// --- UI Views ---

impl KuriUninstaller {
    fn view_loading(&self, message: &str) -> Element<Message, theme::Fluent> {
        column![
            Space::with_height(Length::Fill),
            text(message).size(32),
            text("Please wait.").size(20),
            Space::with_height(Length::Fill),
        ]
        .align_items(iced::Alignment::Center)
        .spacing(20)
        .into()
    }

    fn view_program_list(&self) -> Element<Message, theme::Fluent> {
        let program_list = self.programs.iter().fold(column![].spacing(5), |col, program| {
            let program_clone = program.clone();
            let is_selected = self.selected_program.as_ref() == Some(program);
            let button = button(text(format!("{} ({})", program.name, program.version)))
                .on_press(Message::ProgramSelected(program_clone))
                .style(if is_selected { theme::Button::Primary } else { theme::Button::Secondary })
                .width(Length::Fill);
            col.push(button)
        });

        let scan_button = button(text("Scan for Leftovers")).style(theme::Button::Primary).padding(10);
        let scan_button = if self.selected_program.is_some() {
            scan_button.on_press(Message::ScanButtonPressed)
        } else {
            scan_button // Disabled
        };

        column![
            text("Installed Programs").size(32),
            text(self.selected_program.as_ref().map_or("Select a program to scan", |p| &p.name)).size(20),
            Space::with_height(Length::Fixed(15.0)),
            container(scrollable(program_list)).height(Length::Fill),
            Space::with_height(Length::Fixed(15.0)),
            scan_button,
        ]
        .spacing(20)
        .align_items(iced::Alignment::Center)
        .into()
    }

    fn view_scan_results(&self) -> Element<Message, theme::Fluent> {
        let results_list = self.scan_results.iter().enumerate().fold(
            column![].spacing(5),
            |col, (i, (item, is_checked))| {
                let checkbox = checkbox(item.to_string(), *is_checked)
                    .on_toggle(move |checked| Message::ResultChecked(i, checked));
                col.push(checkbox)
            },
        );

        let back_button = button(text("Back to List")).style(theme::Button::Secondary)
            .on_press(Message::BackButtonPressed).padding(10);
            
        let delete_button = button(text("Delete Selected")).style(theme::Button::Primary).padding(10);
        let delete_button = if self.scan_results.iter().any(|(_, checked)| *checked) {
            delete_button.on_press(Message::DeleteSelectedButtonPressed)
        } else {
            delete_button // Disabled
        };

        let select_all_button = button(text("Select All")).style(theme::Button::Secondary)
            .on_press(Message::SelectAll).padding(5);
        let deselect_all_button = button(text("Deselect All")).style(theme::Button::Secondary)
            .on_press(Message::DeselectAll).padding(5);

        let title = text(format!("Scan Results for {}", self.selected_program.as_ref().unwrap().name)).size(32);

        column![
            title,
            text(format!("Found {} items. Uncheck items to keep them.", self.scan_results.len())).size(16),
            iced::widget::row![select_all_button, deselect_all_button].spacing(10),
            Space::with_height(Length::Fixed(10.0)),
            container(scrollable(results_list)).height(Length::Fill),
            Space::with_height(Length::Fixed(15.0)),
            iced::widget::row![back_button, Space::with_width(Length::Fill), delete_button].spacing(10),
        ]
        .spacing(20)
        .align_items(iced::Alignment::Center)
        .into()
    }

    fn view_confirm_delete(&self) -> Element<Message, theme::Fluent> {
        let items_to_delete_count = self.scan_results.iter().filter(|(_, checked)| *checked).count();

        let confirmation_text = text(format!("Are you sure you want to delete {} selected items?", items_to_delete_count)).size(24);
        let warning_text = text("Files will be moved to the Recycle Bin, but registry keys will be permanently deleted.").size(16);
        
        let backup_checkbox = checkbox("Create a log of registry keys to be deleted", self.backup_registry)
            .on_toggle(Message::BackupCheckboxToggled);

        let confirm_button = button(text("Yes, Delete Them")).style(theme::Button::Primary)
            .on_press(Message::ConfirmDelete).padding(10);
        let cancel_button = button(text("Cancel")).style(theme::Button::Secondary)
            .on_press(Message::CancelDelete).padding(10);

        column![
            Space::with_height(Length::Fill),
            confirmation_text,
            warning_text,
            Space::with_height(Length::Fixed(15.0)),
            backup_checkbox,
            Space::with_height(Length::Fixed(20.0)),
            iced::widget::row![cancel_button, confirm_button].spacing(10),
            Space::with_height(Length::Fill),
        ]
        .spacing(20)
        .align_items(iced::Alignment::Center)
        .into()
    }
}

// --- Core Logic Functions ---

async fn load_installed_programs() -> Result<Vec<ProgramInfo>, String> {
    let mut programs = Vec::new();
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let uninstall_paths = [
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ];

    for path in uninstall_paths {
        if let Ok(uninstall) = hklm.open_subkey(path) {
            for key_name in uninstall.enum_keys().filter_map(Result::ok) {
                if let Ok(subkey) = uninstall.open_subkey(&key_name) {
                    if let Ok(name) = subkey.get_value::<String, _>("DisplayName") {
                        if name.is_empty() { continue; }

                        let version = subkey.get_value("DisplayVersion").unwrap_or_default();
                        let install_location: Option<PathBuf> = subkey
                            .get_value::<String, _>("InstallLocation")
                            .ok()
                            .filter(|s| !s.is_empty())
                            .map(PathBuf::from);
                        
                        if !programs.iter().any(|p: &ProgramInfo| p.name == name) {
                            programs.push(ProgramInfo { name, version, install_location });
                        }
                    }
                }
            }
        }
    }

    programs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(programs)
}

fn generate_search_terms(program: &ProgramInfo) -> Vec<String> {
    let mut terms = vec![program.name.to_lowercase()];
    terms.push(program.name.replace(' ', "").to_lowercase());

    if let Some(location) = &program.install_location {
        if let Some(folder_name) = location.file_name() {
            terms.push(folder_name.to_string_lossy().to_lowercase());
        }
    }
    
    terms.dedup();
    terms
}

async fn scan_for_leftovers(program: ProgramInfo) -> Result<Vec<FoundItem>, String> {
    let mut results = Vec::new();
    let search_terms = generate_search_terms(&program);

    let mut search_dirs = vec![
        dirs::data_local_dir(),
        dirs::data_dir(),
        dirs::config_dir(),
    ]
    .into_iter()
    .filter_map(|p| p)
    .collect::<Vec<_>>();
    
    if let Some(pd) = dirs::data_local_dir() { // Using data_local_dir as a stand-in for ProgramData
        search_dirs.push(pd);
    }

    if let Some(install_loc) = &program.install_location {
        if install_loc.exists() {
            search_dirs.push(install_loc.clone());
        }
    }

    for dir in search_dirs {
        for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let entry_name = entry.file_name().to_string_lossy().to_lowercase();
            if search_terms.iter().any(|term| entry_name.contains(term)) {
                if entry.file_type().is_dir() {
                    results.push(FoundItem::Directory(entry.path().to_path_buf()));
                } else {
                    results.push(FoundItem::File(entry.path().to_path_buf()));
                }
            }
        }
    }

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let reg_paths_to_scan = [
        (&hklm, "SOFTWARE", "HKEY_LOCAL_MACHINE\\SOFTWARE"),
        (&hklm, "SOFTWARE\\Wow6432Node", "HKEY_LOCAL_MACHINE\\SOFTWARE\\Wow6432Node"),
        (&hkcu, "Software", "HKEY_CURRENT_USER\\Software"),
    ];

    for (hive, path_to_open, path_prefix_for_results) in &reg_paths_to_scan {
        if let Ok(base_key) = hive.open_subkey(path_to_open) {
            for subkey_name in base_key.enum_keys().filter_map(Result::ok) {
                 if search_terms.iter().any(|term| subkey_name.to_lowercase().contains(term)) {
                     results.push(FoundItem::RegistryKey(format!("{}\\{}", path_prefix_for_results, subkey_name)));
                 }
            }
        }
    }
    
    results.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
    results.dedup();

    Ok(results)
}

async fn delete_items(items: Vec<FoundItem>, backup: bool) -> Result<(), String> {
    let mut errors = Vec::new();
    let reg_keys_to_delete: Vec<_> = items
        .iter()
        .filter_map(|item| match item {
            FoundItem::RegistryKey(key) => Some(key.clone()),
            _
 => None,
        })
        .collect();

    if backup && !reg_keys_to_delete.is_empty() {
        if let Err(e) = backup_registry_keys(&reg_keys_to_delete).await {
            errors.push(format!("Failed to create registry log: {}", e));
        }
    }

    for item in items {
        match item {
            FoundItem::File(path) | FoundItem::Directory(path) => {
                if let Err(e) = trash::delete(&path) {
                    errors.push(format!("Failed to delete {}: {}", path.display(), e));
                }
            }
            FoundItem::RegistryKey(key_path) => {
                let (hive_str, sub_path) = match key_path.split_once('\\') {
                    Some(parts) => parts,
                    None => {
                        errors.push(format!("Invalid registry path format: {}", key_path));
                        continue;
                    }
                };

                let hive = match hive_str {
                    "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
                    "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
                    _ => {
                        errors.push(format!("Unknown registry hive in path: {}", key_path));
                        continue;
                    }
                };
                
                if let Some((parent_path, key_to_delete)) = sub_path.rsplit_once('\\') {
                    if let Ok(parent_key) = hive.open_subkey_with_flags(parent_path, KEY_WRITE) {
                        if let Err(e) = parent_key.delete_subkey_all(key_to_delete) {
                            errors.push(format!("Failed to delete registry key {}: {}", key_path, e));
                        }
                    } else {
                        errors.push(format!("Could not open parent key for: {}", key_path));
                    }
                } else {
                    if let Err(e) = hive.delete_subkey_all(sub_path) {
                        errors.push(format!("Failed to delete registry key {}: {}", key_path, e));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

async fn backup_registry_keys(keys: &[String]) -> Result<(), String> {
    let backup_dir = dirs::document_dir()
        .ok_or("Could not find Documents directory")?
        .join("KuriUninstaller_Backups");
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let backup_file_path = backup_dir.join(format!("deleted_keys_log-{}.txt", timestamp));
    
    let mut file = fs::File::create(backup_file_path).map_err(|e| e.to_string())?;
    
    writeln!(file, "Log of registry keys deleted by Kuri Uninstaller at {}", timestamp).map_err(|e| e.to_string())?;
    writeln!(file, "--------------------------------------------------").map_err(|e| e.to_string())?;
    
    for key in keys {
        writeln!(file, "{}", key).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}