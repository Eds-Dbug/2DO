use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, NaiveDateTime, Utc, Datelike, Timelike};
use std::path::PathBuf;
use std::fs;

// Calendar file structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarFile {
    pub name: String,
    pub path: String,
    pub last_modified: String,
    pub todo_count: usize,
}

// Todo structure that matches the frontend
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Todo {
    pub id: String, // UID from iCalendar
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub priority: String,
    pub category: Option<String>,
    #[serde(rename = "dueDate")]
    pub due_date: Option<String>, // ISO date string - matches frontend naming
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>, // ISO datetime string - matches frontend naming
    pub calendar_name: String,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Get the calendars directory path for display
#[tauri::command]
fn get_calendars_path() -> Result<String, String> {
    let calendars_dir = get_calendars_dir()?;
    Ok(calendars_dir.to_string_lossy().to_string())
}

// Get the calendars directory path (local to app for USB portability)
fn get_calendars_dir() -> Result<PathBuf, String> {
    // Get the executable path and work backwards to find project root
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;
    
    eprintln!("Executable path: {:?}", exe_path);
    
    // For development, the executable is in target/debug/, so go up to project root
    let mut search_path = exe_path.parent().ok_or("Failed to get parent directory")?.to_path_buf();
    
    eprintln!("Starting search from: {:?}", search_path);
    
    // Look for calendars directory by going up the directory tree
    // Skip the debug directory and go straight to project root
    loop {
        // Skip the debug directory - go up to target first
        if search_path.ends_with("debug") {
            if let Some(parent) = search_path.parent() {
                search_path = parent.to_path_buf();
                eprintln!("Skipping debug directory, moving up to: {:?}", search_path);
                continue;
            }
        }
        
        let calendars_dir = search_path.join("calendars");
        eprintln!("Checking for calendars at: {:?}", calendars_dir);
        
        if calendars_dir.exists() {
            // Check if this calendars directory has any ICS files
            if has_ics_files(&calendars_dir) {
                eprintln!("Found calendars directory with ICS files at: {:?}", calendars_dir);
                return Ok(calendars_dir);
            } else {
                eprintln!("Found empty calendars directory at: {:?}, continuing search", calendars_dir);
            }
        }
        
        // If we can't go up further, break
        if let Some(parent) = search_path.parent() {
            search_path = parent.to_path_buf();
            eprintln!("Moving up to: {:?}", search_path);
        } else {
            eprintln!("No more parents, stopping search");
            break;
        }
        
        // Safety check to avoid infinite loops
        if search_path.components().count() < 3 {
            eprintln!("Reached root directory, stopping search");
            break;
        }
    }
    
    // If we get here, we didn't find any calendars directory with ICS files
    // Create it next to the executable as a fallback
    let app_dir = exe_path.parent().ok_or("Failed to get parent directory")?;
    let calendars_dir = app_dir.join("calendars");
    
    eprintln!("Creating calendars directory at: {:?}", calendars_dir);
    
    // Create directory if it doesn't exist
    if !calendars_dir.exists() {
        fs::create_dir_all(&calendars_dir)
            .map_err(|e| format!("Failed to create calendars directory: {}", e))?;
    }
    
    Ok(calendars_dir)
}

// Helper function to check if a directory contains ICS files
fn has_ics_files(dir: &PathBuf) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ext == "ics" {
                    return true;
                }
            }
        }
    }
    false
}

// List all available calendar files
#[tauri::command]
async fn list_calendars() -> Result<Vec<CalendarFile>, String> {
    let calendars_dir = get_calendars_dir()?;
    let mut calendars = Vec::new();
    
    let entries = fs::read_dir(&calendars_dir)
        .map_err(|e| format!("Failed to read calendars directory: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("ics") {
            let metadata = entry.metadata()
                .map_err(|e| format!("Failed to read file metadata: {}", e))?;
            
            let name = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();
            
            let last_modified = metadata.modified()
                .map_err(|e| format!("Failed to get modification time: {}", e))?
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| format!("Failed to convert modification time: {}", e))?
                .as_secs();
            
            // Count todos in this calendar
            let todo_count = count_todos_in_file(&path).unwrap_or(0);
            
            calendars.push(CalendarFile {
                name,
                path: path.to_string_lossy().to_string(),
                last_modified: last_modified.to_string(),
                todo_count,
            });
        }
    }
    
    // Sort by last modified (newest first)
    calendars.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    
    Ok(calendars)
}

// Count todos in a calendar file
fn count_todos_in_file(path: &PathBuf) -> Result<usize, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let mut count = 0;
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        if lines[i].trim() == "BEGIN:VTODO" {
            count += 1;
            // Skip to the end of this VTODO block
            while i < lines.len() && lines[i].trim() != "END:VTODO" {
                i += 1;
            }
        }
        i += 1;
    }
    
    Ok(count)
}

// Load todos from a specific calendar file
#[tauri::command]
async fn load_todos_from_calendar(calendar_path: String) -> Result<Vec<Todo>, String> {
    let content = fs::read_to_string(&calendar_path)
        .map_err(|e| format!("Failed to read calendar file: {}", e))?;
    
    let calendar_name = PathBuf::from(&calendar_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let mut todos = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    let mut vtodo_count = 0;
    let mut parsed_count = 0;
    
    while i < lines.len() {
        if lines[i].trim() == "BEGIN:VTODO" {
            vtodo_count += 1;
            let mut vtodo_lines = Vec::new();
            i += 1;
            
            // Collect all lines for this VTODO block
            while i < lines.len() && lines[i].trim() != "END:VTODO" {
                vtodo_lines.push(lines[i]);
                i += 1;
            }
            
            match parse_vtodo_from_lines(&vtodo_lines, &calendar_name) {
                Ok(todo) => {
                    todos.push(todo);
                    parsed_count += 1;
                },
                Err(e) => {
                    eprintln!("Failed to parse VTODO {}: {}", vtodo_count, e);
                }
            }
        }
        i += 1;
    }
    
    eprintln!("Parsed {}/{} VTODOs from calendar '{}'", parsed_count, vtodo_count, calendar_name);
    
    Ok(todos)
}

// Parse a VTODO from raw iCalendar lines
fn parse_vtodo_from_lines(lines: &[&str], calendar_name: &str) -> Result<Todo, String> {
    let mut id = String::new();
    let mut title = String::new();
    let mut description = String::new();
    let mut completed = false;
    let mut priority = "medium".to_string();
    let mut category = None;
    let mut due_date = None;
    let mut created_at = None;
    
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        if let Some(colon_pos) = line.find(':') {
            let property_name = &line[..colon_pos];
            let property_value = &line[colon_pos + 1..];
            
            // Handle properties with parameters (e.g., DUE;VALUE=DATE)
            let base_property = if let Some(semicolon_pos) = property_name.find(';') {
                &property_name[..semicolon_pos]
            } else {
                property_name
            };
            
            match base_property {
                "UID" => id = property_value.to_string(),
                "SUMMARY" => title = unescape_ical_text(property_value),
                "DESCRIPTION" => description = unescape_ical_text(property_value),
                "STATUS" => {
                    completed = property_value == "COMPLETED";
                },
                "PRIORITY" => {
                    priority = match property_value {
                        "1" | "2" | "3" => "high",
                        "4" | "5" | "6" => "medium",
                        "7" | "8" | "9" => "low",
                        _ => "medium",
                    }.to_string();
                },
                "CATEGORIES" => {
                    category = Some(unescape_ical_text(property_value));
                },
                "DUE" => {
                    // Parse iCalendar date format (YYYYMMDD or YYYYMMDDTHHMMSSZ)
                    if property_value.len() >= 8 {
                        let date_part = &property_value[0..8];
                        if let Ok(year) = date_part[0..4].parse::<i32>() {
                            if let Ok(month) = date_part[4..6].parse::<u32>() {
                                if let Ok(day) = date_part[6..8].parse::<u32>() {
                                    if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                                        due_date = Some(date.format("%Y-%m-%d").to_string());
                                    }
                                }
                            }
                        }
                    }
                },
                "CREATED" | "DTSTAMP" => {
                    eprintln!("Parsing {} field: '{}' (len: {})", base_property, property_value, property_value.len());
                    // Parse iCalendar datetime format (YYYYMMDDTHHMMSSZ)
                    if property_value.len() >= 15 && property_value.contains('T') {
                        let date_part = &property_value[0..8];
                        let time_part = &property_value[9..15];
                        eprintln!("  Date part: '{}', Time part: '{}'", date_part, time_part);
                        
                        if let Ok(year) = date_part[0..4].parse::<i32>() {
                            if let Ok(month) = date_part[4..6].parse::<u32>() {
                                if let Ok(day) = date_part[6..8].parse::<u32>() {
                                    if let Ok(hour) = time_part[0..2].parse::<u32>() {
                                        if let Ok(minute) = time_part[2..4].parse::<u32>() {
                                            if let Ok(second) = time_part[4..6].parse::<u32>() {
                                                if let Some(dt) = NaiveDate::from_ymd_opt(year, month, day)
                                                    .and_then(|d| d.and_hms_opt(hour, minute, second)) {
                                                    let formatted = dt.format("%Y-%m-%dT%H:%M:%S").to_string();
                                                    eprintln!("  Successfully parsed {} to: '{}'", base_property, formatted);
                                                    created_at = Some(formatted);
                                                } else {
                                                    eprintln!("  Failed to create datetime from {}-{}-{} {}:{}:{}", year, month, day, hour, minute, second);
                                                }
                                            } else {
                                                eprintln!("  Failed to parse second: '{}'", &time_part[4..6]);
                                            }
                                        } else {
                                            eprintln!("  Failed to parse minute: '{}'", &time_part[2..4]);
                                        }
                                    } else {
                                        eprintln!("  Failed to parse hour: '{}'", &time_part[0..2]);
                                    }
                                } else {
                                    eprintln!("  Failed to parse day: '{}'", &date_part[6..8]);
                                }
                            } else {
                                eprintln!("  Failed to parse month: '{}'", &date_part[4..6]);
                            }
                        } else {
                            eprintln!("  Failed to parse year: '{}'", &date_part[0..4]);
                        }
                    } else if property_value.len() == 8 {
                        eprintln!("  Parsing as date-only format");
                        // Handle date-only format (YYYYMMDD)
                        if let Ok(year) = property_value[0..4].parse::<i32>() {
                            if let Ok(month) = property_value[4..6].parse::<u32>() {
                                if let Ok(day) = property_value[6..8].parse::<u32>() {
                                    if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                                        let formatted = date.format("%Y-%m-%d").to_string();
                                        eprintln!("  Successfully parsed {} to: '{}'", base_property, formatted);
                                        created_at = Some(formatted);
                                    } else {
                                        eprintln!("  Failed to create date from {}-{}-{}", year, month, day);
                                    }
                                } else {
                                    eprintln!("  Failed to parse day: '{}'", &property_value[6..8]);
                                }
                            } else {
                                eprintln!("  Failed to parse month: '{}'", &property_value[4..6]);
                            }
                        } else {
                            eprintln!("  Failed to parse year: '{}'", &property_value[0..4]);
                        }
                    } else {
                        eprintln!("  Field length {} is not 8 or >=15, skipping", property_value.len());
                    }
                },
                _ => {} // Ignore other properties
            }
        }
    }
    
    // Generate ID if not present
    if id.is_empty() {
        id = uuid::Uuid::new_v4().to_string();
    }
    
    // Set default title if empty
    if title.is_empty() {
        title = "Untitled Task".to_string();
    }
    
    Ok(Todo {
        id,
        title,
        description,
        completed,
        priority,
        category,
        due_date,
        created_at,
        calendar_name: calendar_name.to_string(),
    })
}

// Save todos back to a calendar file
#[tauri::command]
async fn save_todos_to_calendar(calendar_path: String, todos: Vec<Todo>) -> Result<(), String> {
    let mut calendar_content = String::new();
    
    eprintln!("Saving {} todos to calendar file: {}", todos.len(), calendar_path);
    
    // Start iCalendar header
    calendar_content.push_str("BEGIN:VCALENDAR\r\n");
    calendar_content.push_str("VERSION:2.0\r\n");
    calendar_content.push_str("PRODID:-//Todo Calendar//Todo Calendar//EN\r\n");
    calendar_content.push_str("CALSCALE:GREGORIAN\r\n");
    
    // Add each todo as a VTODO
    for todo in todos {
        calendar_content.push_str("BEGIN:VTODO\r\n");
        calendar_content.push_str(&format!("UID:{}\r\n", todo.id));
        calendar_content.push_str(&format!("SUMMARY:{}\r\n", escape_ical_text(&todo.title)));
        
        if !todo.description.is_empty() {
            calendar_content.push_str(&format!("DESCRIPTION:{}\r\n", escape_ical_text(&todo.description)));
        }
        
        // Status
        if todo.completed {
            calendar_content.push_str("STATUS:COMPLETED\r\n");
        } else {
            calendar_content.push_str("STATUS:NEEDS-ACTION\r\n");
        }
        
        // Priority (convert back to iCalendar format)
        let priority = match todo.priority.as_str() {
            "high" => "1",
            "medium" => "5", 
            "low" => "9",
            _ => "5",
        };
        calendar_content.push_str(&format!("PRIORITY:{}\r\n", priority));
        
        // Category
        if let Some(category) = &todo.category {
            calendar_content.push_str(&format!("CATEGORIES:{}\r\n", escape_ical_text(category)));
        }
        
        // Due date
        if let Some(due_date) = &todo.due_date {
            if let Ok(date) = NaiveDate::parse_from_str(due_date, "%Y-%m-%d") {
                calendar_content.push_str(&format!(
                    "DUE:{:04}{:02}{:02}\r\n",
                    date.year(), date.month(), date.day()
                ));
            }
        }
        
        // Created date
        if let Some(created_at) = &todo.created_at {
            if let Ok(date) = NaiveDate::parse_from_str(created_at, "%Y-%m-%d") {
                calendar_content.push_str(&format!(
                    "CREATED:{:04}{:02}{:02}\r\n",
                    date.year(), date.month(), date.day()
                ));
            } else if let Ok(dt) = NaiveDateTime::parse_from_str(created_at, "%Y-%m-%dT%H:%M:%S") {
                calendar_content.push_str(&format!(
                    "CREATED:{:04}{:02}{:02}T{:02}{:02}{:02}Z\r\n",
                    dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second()
                ));
            }
        }
        
        // Timestamp - use a simple approach to avoid formatting issues
        let now = Utc::now();
        let timestamp = format!("{:04}{:02}{:02}T{:02}{:02}{:02}Z", 
            now.year(), now.month(), now.day(),
            now.hour(), now.minute(), now.second());
        calendar_content.push_str(&format!("DTSTAMP:{}\r\n", timestamp));
        
        calendar_content.push_str("END:VTODO\r\n");
    }
    
    // End iCalendar
    calendar_content.push_str("END:VCALENDAR\r\n");
    
    // Write to file
    eprintln!("Writing calendar content ({} bytes) to file", calendar_content.len());
    fs::write(&calendar_path, calendar_content)
        .map_err(|e| format!("Failed to write calendar file: {}", e))?;
    
    eprintln!("Successfully saved calendar file");
    Ok(())
}

// Helper function to escape text for iCalendar format
fn escape_ical_text(text: &str) -> String {
    text.replace("\\", "\\\\")
        .replace(";", "\\;")
        .replace(",", "\\,")
        .replace("\n", "\\n")
        .replace("\r", "")
        .replace("\0", "") // Remove null characters
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect()
}

// Helper to unescape iCalendar text per RFC 5545
// - \n or \N => newline
// - \; => ;, \, => ,
// - \\ => \
fn unescape_ical_text(text: &str) -> String {
    // First, reduce doubled backslashes to single to normalize over-escaped inputs
    let mut s = text.to_string();
    loop {
        let collapsed = s.replace("\\\\", "\\");
        if collapsed == s { break; }
        s = collapsed;
    }
    s = s.replace("\\n", "\n");
    s = s.replace("\\N", "\n");
    s = s.replace("\\;", ";");
    s = s.replace("\\,", ",");
    s
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_calendars_path, list_calendars, load_todos_from_calendar, save_todos_to_calendar])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
