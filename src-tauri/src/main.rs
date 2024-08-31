#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use rusqlite::{params, Connection, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use tokio::time::{interval, Duration};
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct Feature {
    #[serde(rename = "type")]
    feature_type: String,
    geometry: Geometry,
    properties: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Geometry {
    #[serde(rename = "type")]
    geometry_type: String,
    coordinates: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct Property {
    property_key: String,
    property_value: String,
}

// Function to perform data synchronization
async fn perform_sync() -> Result<(), String> {
    dotenv::dotenv().ok(); // Load environment variables

    // Retrieve API URL and DB file path from environment variables
    let remote_url = env::var("API_URL").unwrap_or_else(|_| "http://localhost:5000/getdata".to_string());
    let db_file = env::var("DB_FILE").unwrap_or_else(|_| "local_data.db".to_string());

    // Fetch remote data
    let client = Client::new();
    let response = client.get(&remote_url).send().await.map_err(|e| format!("Failed to fetch data from remote URL: {}", e))?;
    
    let data = response.text().await.map_err(|e| format!("Failed to read response text: {}", e))?;
    let json_data: Value = serde_json::from_str(&data).map_err(|e| format!("Failed to parse JSON data: {}", e))?;

    // Open SQLite database
    let conn = Connection::open(&db_file).map_err(|e| format!("Failed to open SQLite database: {}", e))?;

    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS features (
            id INTEGER PRIMARY KEY,
            feature_type TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS geometries (
            id INTEGER PRIMARY KEY,
            feature_id INTEGER,
            geometry_type TEXT NOT NULL,
            coordinates TEXT NOT NULL,
            FOREIGN KEY (feature_id) REFERENCES features (id)
        );
        CREATE TABLE IF NOT EXISTS properties (
            id INTEGER PRIMARY KEY,
            feature_id INTEGER,
            property_key TEXT NOT NULL,
            property_value TEXT NOT NULL,
            FOREIGN KEY (feature_id) REFERENCES features (id)
        );
    ").map_err(|e| format!("Failed to create tables: {}", e))?;

    let features = json_data["features"].as_array().ok_or("Invalid GeoJSON format: missing 'features' array")?;
    
    for feature in features {
        let feature_type = feature["type"].as_str().ok_or("Missing 'type' in feature")?.to_string();
        let geometry = &feature["geometry"];
        let geometry_type = geometry["type"].as_str().ok_or("Missing 'type' in geometry")?.to_string();
        let coordinates = serde_json::to_string(&geometry["coordinates"]).map_err(|e| format!("Failed to serialize coordinates: {}", e))?;

        // Check if the feature already exists
        let mut stmt = conn.prepare("
            SELECT f.id FROM features f
            JOIN geometries g ON f.id = g.feature_id
            WHERE f.feature_type = ?1 AND g.geometry_type = ?2 AND g.coordinates = ?3
        ").map_err(|e| format!("Failed to prepare query: {}", e))?;

        let mut rows = stmt.query(params![feature_type, geometry_type, coordinates]).map_err(|e| format!("Failed to execute query: {}", e))?;

        if rows.next().map_err(|e| format!("Failed to fetch query result: {}", e))?.is_some() {
            continue; // Skip insertion if the record exists
        }

        conn.execute(
            "INSERT INTO features (feature_type) VALUES (?1)",
            params![feature_type],
        ).map_err(|e| format!("Failed to insert feature: {}", e))?;
        let feature_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO geometries (feature_id, geometry_type, coordinates) VALUES (?1, ?2, ?3)",
            params![feature_id, geometry_type, coordinates],
        ).map_err(|e| format!("Failed to insert geometry: {}", e))?;

        if let Some(props) = feature["properties"].as_object() {
            for (key, value) in props {
                conn.execute(
                    "INSERT INTO properties (feature_id, property_key, property_value) VALUES (?1, ?2, ?3)",
                    params![feature_id, key, value.to_string()],
                ).map_err(|e| format!("Failed to insert property: {}", e))?;
            }
        }
    }

    Ok(())
}

// Tauri command to trigger synchronization
#[tauri::command]
async fn sync_data() -> Result<(), String> {
    perform_sync().await
}

// Tauri command to get synced data
#[tauri::command]
fn get_synced_data() -> Result<serde_json::Value, String> {
    let db_file = env::var("DB_FILE").unwrap_or_else(|_| "local_data.db".to_string());

    let conn = Connection::open(&db_file).map_err(|e| format!("Failed to open SQLite database: {}", e))?;

    let mut stmt = conn.prepare("
        SELECT features.id, features.feature_type, geometries.geometry_type, geometries.coordinates, properties.property_key, properties.property_value
        FROM features
        LEFT JOIN geometries ON features.id = geometries.feature_id
        LEFT JOIN properties ON features.id = properties.feature_id
    ").map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "feature_type": row.get::<_, String>(1)?,
            "geometry_type": row.get::<_, String>(2)?,
            "coordinates": row.get::<_, String>(3)?,
            "property_key": row.get::<_, Option<String>>(4).unwrap_or_default(),
            "property_value": row.get::<_, Option<String>>(5).unwrap_or_default()
        }))
    }).map_err(|e| format!("Failed to query data: {}", e))?.collect::<Result<Vec<_>, _>>().map_err(|e| format!("Failed to collect query results: {}", e))?;

    let mut features_map = HashMap::new();

    for row in rows {
        let id = row["id"].as_i64().ok_or("Invalid feature ID")?;
        let feature_type = row["feature_type"].as_str().ok_or("Invalid feature type")?.to_string();
        let geometry_type = row["geometry_type"].as_str().ok_or("Invalid geometry type")?.to_string();
        let coordinates = row["coordinates"].as_str().ok_or("Invalid coordinates")?.to_string();
        let property_key = row["property_key"].as_str().unwrap_or_default().to_string();
        let property_value = row["property_value"].as_str().unwrap_or_default().to_string();

        let feature = features_map.entry(id).or_insert_with(|| {
            serde_json::json!({
                "type": "Feature",
                "geometry": {
                    "type": geometry_type,
                    "coordinates": serde_json::from_str::<serde_json::Value>(&coordinates).unwrap_or(serde_json::json!([]))
                },
                "properties": {}
            })
        });

        if !property_key.is_empty() {
            feature["properties"][property_key] = serde_json::json!(property_value);
        }
    }

    let features = features_map.into_iter().map(|(_, feature)| feature).collect::<Vec<_>>();

    let geojson = serde_json::json!({
        "type": "FeatureCollection",
        "features": features
    });

    Ok(geojson)
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok(); // Load environment variables

    let sync_interval_seconds: u64 = env::var("SYNC_INTERVAL_SECONDS").unwrap_or_else(|_| "3600".to_string())
        .parse().unwrap_or(3600);

    let db_file = env::var("DB_FILE").unwrap_or_else(|_| "local_data.db".to_string());
    if !std::path::Path::new(&db_file).exists() {
        let conn = Connection::open(&db_file).expect("Failed to open database");
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS features (
                id INTEGER PRIMARY KEY,
                feature_type TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS geometries (
                id INTEGER PRIMARY KEY,
                feature_id INTEGER,
                geometry_type TEXT NOT NULL,
                coordinates TEXT NOT NULL,
                FOREIGN KEY (feature_id) REFERENCES features (id)
            );
            CREATE TABLE IF NOT EXISTS properties (
                id INTEGER PRIMARY KEY,
                feature_id INTEGER,
                property_key TEXT NOT NULL,
                property_value TEXT NOT NULL,
                FOREIGN KEY (feature_id) REFERENCES features (id)
            );
        ").expect("Failed to create tables");
    }

    let sync_interval = Arc::new(sync_interval_seconds);
    let sync_interval_clone = sync_interval.clone();

    let sync_task = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(*sync_interval_clone));
        loop {
            interval.tick().await;
            if let Err(e) = perform_sync().await {
                eprintln!("Error during sync: {}", e);
            }
        }
    });

    tauri::Builder::default()
        .manage(sync_task)
        .invoke_handler(tauri::generate_handler![sync_data, get_synced_data])
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
