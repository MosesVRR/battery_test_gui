use crate::pilot::{BatteryBench, BatteryBenchState, CompletionStatus};
use rusqlite::{params, Connection, Result};
use chrono::{DateTime, Utc};

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use csv::Writer;



/// Generates a CSV file for each battery ID from the SQLite database "battery_logs.db".
///
/// # Arguments
///
/// * `conn` - A reference to the SQLite database connection.
/// * `base_path` - The base path where the CSV files will be saved.
///
/// # Returns
///
/// A `Result<(), String>` indicating success or failure.
pub fn export_to_csv(conn: &Connection, base_path: &str) -> Result<(), String> {
    // Prepare the SQL query to select all rows from the battery_logs table.
    let mut stmt = conn.prepare("SELECT * FROM battery_logs")
        .map_err(|err| format!("Failed to prepare query: {}", err))?;
    
    // Query the database and map rows to a tuple.
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?, // record_id
            row.get::<_, i64>(1)?, // id
            row.get::<_, String>(2)?, // port
            row.get::<_, i64>(3)?, // temperature
            row.get::<_, i64>(4)?, // battery_temperature
            row.get::<_, i64>(5)?, // electronic_load_temperature
            row.get::<_, i64>(6)?, // voltage
            row.get::<_, i64>(7)?, // current
            row.get::<_, String>(8)?, // state
            row.get::<_, String>(9)?, // status
            row.get::<_, String>(10)?, // start_date
            row.get::<_, String>(11)?, // end_date
        ))
    }).map_err(|err| format!("Failed to query database: {}", err))?;
    
    // Create a HashMap to group rows by battery ID.
    let mut battery_logs: std::collections::HashMap<i64, Vec<(i64, i64, String, i64, i64, i64, i64, i64, String, String, String, String)>> = std::collections::HashMap::new();
    
    for row in rows {
        let row = row.map_err(|err| format!("Failed to fetch row: {}", err))?;
        battery_logs.entry(row.1).or_insert_with(Vec::new).push(row);
    }
    
    // Iterate over each battery ID and create a CSV file.
    for (battery_id, logs) in battery_logs {
        let csv_filename = format!("battery_{}.csv", battery_id);
        let mut path = PathBuf::from(base_path);
        path.push(csv_filename);

        // Create a CSV writer that writes to the specified file path.
        let mut wtr = Writer::from_path(&path)
            .map_err(|err| format!("Failed to create CSV writer: {}", err))?;

        // Write the header record to the CSV file.
        wtr.write_record(&["record_id", "id", "port", "temperature", "battery_temperature", "electronic_load_temperature", "voltage", "current", "state", "status", "start_date", "end_date"])
            .map_err(|err| format!("Failed to write CSV header: {}", err))?;

        // Write each record to the CSV file.
        for log in logs {
            wtr.write_record(&[
                log.0.to_string(),
                log.1.to_string(),
                log.2,
                log.3.to_string(),
                log.4.to_string(),
                log.5.to_string(),
                log.6.to_string(),
                log.7.to_string(),
                log.8,
                log.9,
                log.10,
                log.11,
            ]).map_err(|err| format!("Failed to write CSV record: {}", err))?;
        }

        // Flush the writer to ensure all data is written to the file.
        wtr.flush().map_err(|err| format!("Failed to flush CSV writer: {}", err))?;
    }

    Ok(())
}

/// Logs all values associated to a battery bench to a databse file.
///
/// The values are logged into the database named after the ID of the battery.
///
/// # Arguments
///
/// * `conn` - A reference to the SQLite database connection.
/// * `battery_bench` - A BatteryBench object holding information about a battery bench.
///
/// # Returns
///
/// A `Result<(), &'static str>` an error message if the logging failed.
///
pub fn log_battery(conn: &Connection, battery_bench: BatteryBench) -> Result<(), &'static str> {
    // Open or create the SQLite database
    //let conn = Connection::open("battery_logs.db").map_err(|_| "Failed to open database")?;

    // Convert enums to their corresponding string representations
    let state_str = match battery_bench.state {
        crate::pilot::BatteryBenchState::Standby => "Standby",
        crate::pilot::BatteryBenchState::Charge => "Charge",
        crate::pilot::BatteryBenchState::Discharge => "Discharge",
    };

    let status_str = match battery_bench.status {
        crate::pilot::CompletionStatus::Success => "Success",
        crate::pilot::CompletionStatus::Fail => "Fail",
        crate::pilot::CompletionStatus::InProgress => "InProgress",
    };

    // Execute the SQL INSERT statement
    conn.execute(
        "INSERT INTO battery_logs (
            id, port, temperature, battery_temperature,
            electronic_load_temperature, voltage, current,
            state, status, start_date, end_date
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            battery_bench.id,
            battery_bench.port,
            battery_bench.temperature,
            battery_bench.battery_temperature,
            battery_bench.electronic_load_temperature,
            battery_bench.voltage,
            battery_bench.current,
            state_str,
            status_str,
            battery_bench.start_date.to_rfc3339(),
            battery_bench.end_date.to_rfc3339(),
        ],
    ).map_err(|err| {
		eprintln!("Failed to insert data into database: {}", err);
		"Failed to insert data into database"
	})?;

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::OpenFlags;

    #[test]
    fn test_log_battery() {
        // Setup: Initialize an in-memory SQLite database connection
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");

		// Create the table (if it doesn't exist)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS battery_logs (
                record_id INTEGER PRIMARY KEY AUTOINCREMENT,
                id INTEGER NOT NULL,
                port TEXT NOT NULL,
                temperature INTEGER NOT NULL,
                battery_temperature INTEGER NOT NULL,
                electronic_load_temperature INTEGER NOT NULL,
                voltage INTEGER NOT NULL,
                current INTEGER NOT NULL,
                state TEXT NOT NULL,
                status TEXT NOT NULL,
                start_date TEXT,
                end_date TEXT
            )",
            [],
        ).expect("Failed to create table");

        // Example BatteryBench object for testing
        let battery_bench = BatteryBench {
            id: 99,
            port: "COM4".to_string(),
            temperature: 25,
            battery_temperature: 30,
            electronic_load_temperature: 27,
            voltage: 12,
            current: 1,
            state: BatteryBenchState::Charge,
            status: CompletionStatus::InProgress,
            start_date: Utc::now(),
            end_date: Utc::now(),
        };

        // Exercise: Call the function under test
        let result = log_battery(&conn, battery_bench.clone());  // Only pass the BatteryBench object

        // Assertion: Check the result
    	assert!(result.is_ok(), "Failed to log battery data: {:?}", result.err());

    	// Verification: Query the database to verify the inserted data
    	let mut stmt = conn.prepare("SELECT * FROM battery_logs WHERE id = ?1").expect("Failed to prepare statement");
    	let rows = stmt.query_map(params![battery_bench.id], |row| {
        	Ok(())
    	}).expect("Failed to query database");

    	for row in rows {
        	println!("Row found: {:?}", row);
    	}
    }
}