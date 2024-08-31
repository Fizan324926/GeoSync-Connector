# GeoSync Connector

This Tauri application synchronizes GeoJSON data from a remote API with a local SQLite database. The app periodically fetches GeoJSON data from the API, stores it in a local SQLite database, and provides an interface for retrieving the synchronized data.

## Features

- **Periodic Data Synchronization**: Automatically fetches data from a specified API URL at defined intervals and stores it in a local SQLite database.
- **SQLite Database Management**: Manages GeoJSON data efficiently with structured tables for features, geometries, and properties.
- **Tauri Commands**: 
  - `sync_data`: Manually trigger the data synchronization process.
  - `get_synced_data`: Retrieve all synchronized data from the local SQLite database in GeoJSON format.
- **Error Handling**: Detailed error messages for synchronization failures or data retrieval issues.

## Prerequisites

1. **Rust & Cargo**: [Install Rust](https://www.rust-lang.org/tools/install).
2. **Node.js & npm**: [Install Node.js](https://nodejs.org/).
3. **Tauri CLI**: Install with `cargo install tauri-cli`.
4. **Install npm Packages**: Run `npm install` in the project directory.
5. **Set Environment**: Create a `.env` file with required variables (API_URL, DB_FILE, SYNC_INTERVAL_SECONDS).

## Installation

1. **Clone the repository**:
    ```sh
    git clone https://github.com/Fizan324926/GeoSync-Connector.git
    cd GeoSync-Connector
    ```

2. **Build the project**:
    ```sh
    npm install
    npm run build
    npm run tauri:build
    ```

   This will compile the project and produce an executable in the `src-tauri/target/release` directory.
   On Windows, it will be a .exe file.
   On macOS, it will be a .app bundle.
   On Linux, it will be a binary file.

## Usage

### Setup Environment Variables
  Create a .env file in the  root installation dir with the following variables:
   ```sh
   API_URL=http://localhost:5000/getdata # Replace with your actual API URL
   DB_FILE=local_data.db # Path to the SQLite database file
   SYNC_INTERVAL_SECONDS=3600 # Sync interval in seconds (default is 3600 seconds or 1 hour)
   ```
### Running the Example Flask API

The project includes an example Flask API that serves data from the `sample_new_york_parcels` dataset you provided. Follow the steps below to run the API:

1. **Navigate to the `test-api-server` directory:**

   ```sh
   cd test-api-server
   ```
2. **Run the Flask API using Python:**
   ```sh
   py app.py
   ```
   This will start the API server on http://localhost:5000/getdata, which serves the sample data for testing the GeoJSON synchronization functionality of your application.
   
3. **Configuration:**
   Ensure that the API_URL in your .env file points to this address:
   ```sh
   API_URL=http://localhost:5000/getdata
   ```

## Executing the Application

After building the project, you can run the executable or relevant file for your operating system. Below are the instructions for various platforms:

### For Windows

1. **Locate the Executable:**
   The generated executable file (e.g., `app.exe`) will be in the ``src-tauri/target/release` directory, depending on your build configuration.

2. **Run the Executable:**
   You can execute the application by double-clicking the `.exe` file or running it via the command line:

   ```sh
   ./app.exe
   ```


