from flask import Flask, jsonify, abort
import os

app = Flask(__name__)

@app.route('/getdata', methods=['GET'])
def get_geojson_data():
    file_path = 'data.geojson'
    app.logger.info(f"Attempting to read file: {file_path}")

    if not os.path.exists(file_path):
        app.logger.error(f"File not found: {file_path}")
        abort(404, description="File not found")

    try:
        # Read the contents of the GeoJSON file
        with open(file_path, 'r') as file:
            data = file.read()
        app.logger.info(f"File read successfully. Data: {data[:100]}")  # Log first 100 chars
        return data, 200, {'Content-Type': 'application/json'}
    except Exception as e:
        app.logger.error(f"Error reading file: {str(e)}")
        abort(500, description=f"Error reading file: {str(e)}")

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000, debug=True)
