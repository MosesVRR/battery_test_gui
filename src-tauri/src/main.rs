// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod file;
mod pilot;
 mod serial;
mod database;
mod commands;
mod uart;

use std::fs::File; // addedd for debugging - to be removed/commented when building
use std::io::Write; // same as above
use std::thread;
use std::time::Duration;

use chrono::Utc;
use tauri::Manager;
use commands::*;

use self::file::*;
use self::pilot::*;
use self::serial::*;
use self::database::initialize_database;
use self::uart::uart_read;



// For Debugging - to be removed/commented when building
// fn get_writable_path() -> PathBuf {
//     // Modify this as needed
//     let mut path = PathBuf::from("C:\\Users\\zephr\\Desktop\\SC");
//     path.push("export.csv");
//     path
// }

fn main() {
    // Initialize the database
    let conn = initialize_database().expect("Failed to initialize database");
    
    
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
            thread::spawn(move || {// change COM port number as necessary
                if let Err(e) = uart_read("COM3") {
                    eprintln!("Failed to listen to UART: {:?}", e);
                }
            });
            thread::spawn(move || {
                loop {
                    let battery_bench = BatteryBench {
                        id: 0,
                        port: "COM4".to_string(),
                        temperature: 2020,
                        battery_temperature: 2013,
                        electronic_load_temperature: 2054,
                        voltage: 534,
                        current: 324,
                        state: BatteryBenchState::Standby,
                        status: CompletionStatus::InProgress,
                        start_date: Utc::now(),
                        end_date: Utc::now(),
                    };
            
                    // Log battery data
                    if let Err(e) = log_battery(&conn, battery_bench.clone()) {
                        eprintln!("Failed to log battery data: {}", e);
                    }
            
                    // Emit battery data to frontend
                    app_handle.emit_all("display-battery", battery_bench).unwrap();
                    
                    thread::sleep(Duration::from_secs(1)); // Adjust the interval as needed
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::export_csv_command,
             get_project_dir,
             select_directory             
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    
}
