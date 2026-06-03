#!/usr/bin/env node
const http = require('http');

const port = process.env.API_PORT || process.env.PORT || '8080';
const url = `http://localhost:${port}/api/power`;

console.log(`[INFO] [MONITOR] System Load Monitor started, querying API at ${url}...`);

function checkStatus() {
    http.get(url, (res) => {
        let rawData = '';
        res.on('data', (chunk) => { rawData += chunk; });
        res.on('end', () => {
            try {
                const parsedData = JSON.parse(rawData);
                console.log(`[MONITOR] [${new Date().toISOString()}] Battery: ${parsedData.battery_level}%, Mode: ${parsedData.power_mode}`);
            } catch (e) {
                console.error(`[MONITOR] Error parsing JSON response: ${e.message}`);
            }
        });
    }).on('error', (e) => {
        console.error(`[MONITOR] Request failed: ${e.message}`);
    });
}

// Check status once
checkStatus();

// Keep running every 3 seconds unless TEST_ONCE is active
if (process.env.TEST_ONCE !== 'true') {
    const interval = setInterval(checkStatus, 3000);
    // Graceful shutdown on SIGTERM/SIGINT
    process.on('SIGTERM', () => {
        clearInterval(interval);
        console.log('[INFO] [MONITOR] Monitor shutting down.');
    });
    process.on('SIGINT', () => {
        clearInterval(interval);
        console.log('[INFO] [MONITOR] Monitor shutting down.');
    });
}
