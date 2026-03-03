const express = require('express');
const router = express.Router();

// App version configuration
// Update these values when releasing new versions
const APP_VERSION = {
    current: "1.0.0",
    minimum: "0.1.0",  // Minimum supported version (force update if below this)
    releaseDate: "2026-02-08",
    releaseNotes: `
## What's New in v1.0.0

- 🚀 Initial public release
- 📄 PDF and image receipt extraction
- 🤖 AI-powered data extraction with 5 provider options
- 📊 Export to CSV, Excel, and JSON
- ✨ Custom field support with AI prompt enhancement
- 🔐 License-based activation with offline support
    `.trim()
};

// Download URLs for each platform
const DOWNLOAD_URLS = {
    linux: process.env.DOWNLOAD_URL_LINUX || "https://your-server.com/downloads/receipt_extractor_linux",
    windows: process.env.DOWNLOAD_URL_WINDOWS || "https://your-server.com/downloads/receipt_extractor_windows.exe",
    macos: process.env.DOWNLOAD_URL_MACOS || "https://your-server.com/downloads/receipt_extractor_macos"
};

// Parse semantic version string to comparable array
function parseVersion(version) {
    const parts = version.split('.').map(p => parseInt(p, 10) || 0);
    while (parts.length < 3) parts.push(0);
    return parts;
}

// Compare two versions: returns -1 if a < b, 0 if a == b, 1 if a > b
function compareVersions(a, b) {
    const aParts = parseVersion(a);
    const bParts = parseVersion(b);
    
    for (let i = 0; i < 3; i++) {
        if (aParts[i] < bParts[i]) return -1;
        if (aParts[i] > bParts[i]) return 1;
    }
    return 0;
}

// GET /api/version
// Returns latest version info
router.get('/', async (req, res) => {
    try {
        const clientVersion = req.query.current_version;
        const platform = req.query.platform || 'linux';
        
        const response = {
            latest_version: APP_VERSION.current,
            minimum_version: APP_VERSION.minimum,
            release_date: APP_VERSION.releaseDate,
            release_notes: APP_VERSION.releaseNotes,
            download_url: DOWNLOAD_URLS[platform] || DOWNLOAD_URLS.linux
        };
        
        // If client provided their version, include update info
        if (clientVersion) {
            response.update_available = compareVersions(clientVersion, APP_VERSION.current) < 0;
            response.force_update = compareVersions(clientVersion, APP_VERSION.minimum) < 0;
        }
        
        console.log(`📦 Version check: client=${clientVersion || 'unknown'}, latest=${APP_VERSION.current}, update=${response.update_available || false}`);
        
        res.json(response);
        
    } catch (error) {
        console.error('Version check error:', error);
        res.status(500).json({ error: 'Internal server error' });
    }
});

// GET /api/version/changelog
// Returns full changelog (optional endpoint)
router.get('/changelog', async (req, res) => {
    try {
        res.json({
            versions: [
                {
                    version: "1.0.0",
                    date: "2026-02-08",
                    notes: APP_VERSION.releaseNotes
                }
                // Add more versions here as you release them
            ]
        });
    } catch (error) {
        console.error('Changelog error:', error);
        res.status(500).json({ error: 'Internal server error' });
    }
});

module.exports = router;

