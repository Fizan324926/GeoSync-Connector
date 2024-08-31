import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api';
import { MapContainer, TileLayer, GeoJSON } from 'react-leaflet';
import 'leaflet/dist/leaflet.css';
import './App.css';

function App() {
  const [status, setStatus] = useState('');
  const [geojsonData, setGeojsonData] = useState(null);
  const [stats, setStats] = useState('');
  const [lastFetchedAt, setLastFetchedAt] = useState('');
  const [loading, setLoading] = useState(false); // Add loading state

  // Fetch data when the component mounts
  useEffect(() => {
    const initializeData = async () => {
      setStatus('Fetching data...');
      try {
        const result = await invoke('get_synced_data');
        console.log('Fetched data:', result);

        // Check if result is in expected GeoJSON format
        if (result && result.type === 'FeatureCollection') {
          console.log('GeoJSON data is valid:', result);
          setGeojsonData(result); // Update GeoJSON data
          updateStats(result); // Update stats
          setLastFetchedAt(new Date().toLocaleString()); // Set timestamp of last fetch
        } else {
          console.error('Invalid GeoJSON data:', result);
        }

        setStatus(`Data fetched successfully at ${new Date().toLocaleString()}`);
      } catch (error) {
        console.error('Error fetching data:', error);
        setStatus(`Error fetching data: ${error.message}`);
      }
    };

    initializeData();
  }, []);

  // Sync data and fetch it again
  const syncData = async () => {
    setStatus('Syncing data...');
    setLoading(true); // Set loading to true when starting sync
    try {
      await invoke('sync_data');
      setStatus('Data synchronized successfully!');
      fetchData();
    } catch (error) {
      console.error('Error syncing data:', error);
      setStatus(`Error: ${error.message}`);
    } finally {
      setLoading(false); // Set loading to false when sync is complete
    }
  };

  // Fetch data from the backend
  const fetchData = async () => {
    setStatus('Fetching data...');
    try {
      const result = await invoke('get_synced_data');
      console.log('Fetched data:', result);

      // Check if result is in expected GeoJSON format
      if (result && result.type === 'FeatureCollection') {
        console.log('GeoJSON data is valid:', result);
        setGeojsonData(result); // Update GeoJSON data
        updateStats(result); // Update stats
        setLastFetchedAt(new Date().toLocaleString()); // Set timestamp of last fetch
      } else {
        console.error('Invalid GeoJSON data:', result);
      }

      setStatus(`Data fetched successfully at ${new Date().toLocaleString()}`);
    } catch (error) {
      console.error('Error fetching data:', error);
      setStatus(`Error fetching data: ${error.message}`);
    }
  };

  // Function to update stats based on GeoJSON data
  const updateStats = (data) => {
    if (!data || data.type !== 'FeatureCollection') return;

    let statsText = `Type: ${data.type}`;
    const featureTypes = {};

    data.features.forEach(feature => {
      const type = feature.geometry.type;
      featureTypes[type] = (featureTypes[type] || 0) + 1;
    });

    const featureTypeStats = Object.entries(featureTypes)
      .map(([type, count]) => `${type}: ${count}`)
      .join(', ');

    statsText += ` | Features: ${data.features.length}`;
    if (Object.keys(featureTypes).length > 1) {
      statsText += ` | ${featureTypeStats}`;
    }

    setStats(statsText);
  };

  return (
    <div className="App">
      <h1>GeoSync Connector</h1>
      <button onClick={syncData} disabled={loading}>Sync Data</button>
      <p>{status}</p>
      {loading && <p>Loading...</p>} {/* Display loading message */}
      <div className="container">
        <div id="map">
          <MapContainer center={[40.7128, -74.0060]} zoom={12} style={{ height: "100%", width: "100%" }}>
            <TileLayer
              url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
              attribution="© OpenStreetMap contributors"
            />
            {geojsonData && <GeoJSON data={geojsonData} />}
          </MapContainer>
        </div>
        <div className="data-panel">
          <h2>GeoJSON Data (JSON View)</h2>
          <div className="stats-line">{stats}</div> {/* Display stats with styling */}
          <pre id="geojson-data">{JSON.stringify(geojsonData, null, 2)}</pre>
        </div>
      </div>
    </div>
  );
}

export default App;
