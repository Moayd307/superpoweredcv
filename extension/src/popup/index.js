/**
 * @file popup/index.js
 * @description Popup script for the extension. Handles user interaction.
 */

document.addEventListener('DOMContentLoaded', async () => {
    const grabBtn = document.getElementById('grabBtn');
    const statusDiv = document.getElementById('status');
    const previewDiv = document.getElementById('preview');
    const tabs = document.querySelectorAll('.tab-btn');
    const contents = document.querySelectorAll('.tab-content');
    const historyList = document.getElementById('historyList');
    const clearHistoryBtn = document.getElementById('clearHistoryBtn');
    const autoDownloadCheckbox = document.getElementById('autoDownload');
    const debugModeCheckbox = document.getElementById('debugMode');

    // Load Settings
    try {
        const settings = await loadSettings();
        if (autoDownloadCheckbox) autoDownloadCheckbox.checked = settings.autoDownload;
        if (debugModeCheckbox) debugModeCheckbox.checked = settings.debugMode;
    } catch (e) {
        console.error("Failed to load settings:", e);
    }

    // Tab Switching
    tabs.forEach(tab => {
        tab.addEventListener('click', () => {
            tabs.forEach(t => t.classList.remove('active'));
            contents.forEach(c => c.classList.remove('active'));
            
            tab.classList.add('active');
            document.getElementById(tab.dataset.tab).classList.add('active');
            
            if (tab.dataset.tab === 'history') {
                renderHistory();
            }
        });
    });

    // Settings Listeners
    if (autoDownloadCheckbox) {
        autoDownloadCheckbox.addEventListener('change', () => {
            saveSettings({ autoDownload: autoDownloadCheckbox.checked });
        });
    }
    if (debugModeCheckbox) {
        debugModeCheckbox.addEventListener('change', () => {
            saveSettings({ debugMode: debugModeCheckbox.checked });
        });
    }

    // History Listeners
    if (clearHistoryBtn) {
        clearHistoryBtn.addEventListener('click', async () => {
            await chrome.storage.local.set({ history: [] });
            renderHistory();
        });
    }

    // Listen for progress messages from content script
    chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
        if (request.action === 'progress') {
            updateStatus(statusDiv, request.message);
        }
    });

    if (grabBtn) {
        grabBtn.addEventListener('click', async () => {
            await handleGrabClick(statusDiv, previewDiv);
        });
    }
});

/**
 * Handles the click event on the Grab button.
 * @param {HTMLElement} statusDiv - The status display element.
 * @param {HTMLElement} previewDiv - The preview display element.
 */
async function handleGrabClick(statusDiv, previewDiv) {
    try {
        const settings = await loadSettings();
        const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
        
        if (!tab || !tab.url.includes('linkedin.com/in/')) {
            updateStatus(statusDiv, 'Not a LinkedIn profile page.');
            return;
        }

        updateStatus(statusDiv, 'Scraping...');
        if (previewDiv) previewDiv.innerHTML = ''; // Clear previous preview
        Logger.info('Sending scrape command to tab', tab.id);

        const response = await chrome.tabs.sendMessage(tab.id, { action: 'scrape' });
        
        if (response && response.data) {
            Logger.info('Scrape successful', response.data);
            
            // Render preview using our "templating engine"
            if (previewDiv) {
                previewDiv.innerHTML = renderProfileTemplate(response.data);
            }

            const filename = generateFilename(response.data.name);
            
            // Save to history
            await saveToHistory(response.data, filename);

            // Auto download if enabled
            if (settings.autoDownload) {
                downloadJSON(response.data, filename);
            }
            
            updateStatus(statusDiv, 'Done!');
        } else if (response && response.error) {
            Logger.error('Scrape error from content script', response.error);
            updateStatus(statusDiv, 'Failed: ' + response.error);
        } else {
            Logger.error('Scrape failed, no data');
            updateStatus(statusDiv, 'Failed to scrape.');
        }
    } catch (error) {
        Logger.error('Popup error', error);
        if (error.message.includes('Could not establish connection')) {
            updateStatus(statusDiv, 'Error: Connection failed. Please reload the LinkedIn page and try again.');
        } else {
            updateStatus(statusDiv, 'Error: ' + error.message);
        }
    }
}

async function loadSettings() {
    const result = await chrome.storage.sync.get(['autoDownload', 'debugMode']);
    return {
        autoDownload: result.autoDownload !== false, // default true
        debugMode: result.debugMode || false
    };
}

async function saveSettings(newSettings) {
    await chrome.storage.sync.set(newSettings);
}

async function saveToHistory(profile, filename) {
    const result = await chrome.storage.local.get(['history']);
    const history = result.history || [];
    const entry = {
        name: profile.name,
        headline: profile.headline,
        date: new Date().toISOString(),
        filename: filename,
        data: profile // Store full data for re-download
    };
    // Add to top, limit to 50
    history.unshift(entry);
    if (history.length > 50) history.pop();
    await chrome.storage.local.set({ history });
}

async function renderHistory() {
    const historyList = document.getElementById('historyList');
    if (!historyList) return;
    
    const result = await chrome.storage.local.get(['history']);
    const history = result.history || [];
    
    historyList.innerHTML = '';
    if (history.length === 0) {
        historyList.innerHTML = '<div class="history-item">No history yet.</div>';
        return;
    }

    history.forEach(item => {
        const div = document.createElement('div');
        div.className = 'history-item';
        div.innerHTML = `
            <div>
                <strong>${escapeHtml(item.name)}</strong><br>
                <span class="history-date">${new Date(item.date).toLocaleString()}</span>
            </div>
            <button class="secondary-btn" style="width: auto; padding: 2px 5px;">⬇</button>
        `;
        div.querySelector('button').addEventListener('click', (e) => {
            e.stopPropagation();
            downloadJSON(item.data, item.filename);
        });
        historyList.appendChild(div);
    });
}

/**
 * Generates the filename in the format: superpoweredcv-name-date-time.json
 * @param {string} name - The profile name.
 * @returns {string} The formatted filename.
 */
function generateFilename(name) {
    const cleanName = (name || 'unknown').toLowerCase().replace(/\s+/g, '');
    const now = new Date();
    const dateStr = now.toISOString().split('T')[0]; // YYYY-MM-DD
    const timeStr = now.toTimeString().split(' ')[0].replace(/:/g, '-'); // HH-MM-SS
    return `superpoweredcv-${cleanName}-${dateStr}-${timeStr}.json`;
}

/**
 * Simple templating engine to render profile preview.
 * @param {Object} data - The profile data.
 * @returns {string} HTML string.
 */
function renderProfileTemplate(data) {
    const { name, headline, location, experience, education } = data;
    
    const expCount = experience ? experience.length : 0;
    const eduCount = education ? education.length : 0;

    return `
        <div class="preview-item">
            <span class="preview-key">NAME:</span> ${escapeHtml(name)}
        </div>
        <div class="preview-item">
            <span class="preview-key">HEADLINE:</span> ${escapeHtml(headline).substring(0, 50)}...
        </div>
        <div class="preview-item">
            <span class="preview-key">LOC:</span> ${escapeHtml(location)}
        </div>
        <div class="preview-item">
            <span class="preview-key">STATS:</span> ${expCount} EXP / ${eduCount} EDU
        </div>
    `;
}

/**
 * Escapes HTML characters to prevent XSS.
 * @param {string} str - The string to escape.
 * @returns {string} Escaped string.
 */
function escapeHtml(str) {
    if (!str) return '';
    return str
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
}


/**
 * Updates the status text.
 * @param {HTMLElement} element - The status element.
 * @param {string} text - The text to display.
 */
function updateStatus(element, text) {
    if (element) {
        element.textContent = text;
    }
}

/**
 * Triggers a download of the scraped data as a JSON file.
 * @param {Object} data - The profile data to download.
 * @param {string} filename - The name of the file to save.
 */
function downloadJSON(data, filename) {
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}
