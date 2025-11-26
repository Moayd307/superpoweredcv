/* global Logger */
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
    // historyList is used in renderHistory, not here
    const clearHistoryBtn = document.getElementById('clearHistoryBtn');
    const clearDataBtn = document.getElementById('clearDataBtn');
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
    if (clearDataBtn) {
        clearDataBtn.addEventListener('click', async () => {
            if (confirm('Are you sure you want to delete ALL extension data? This cannot be undone.')) {
                await chrome.storage.local.clear();
                await chrome.storage.sync.clear();
                // Reset UI
                if (autoDownloadCheckbox) autoDownloadCheckbox.checked = true;
                if (debugModeCheckbox) debugModeCheckbox.checked = false;
                alert('All data cleared.');
            }
        });
    }

    // History Listeners
    if (clearHistoryBtn) {
        clearHistoryBtn.addEventListener('click', async () => {
            if (confirm('Clear all history?')) {
                await chrome.storage.local.set({ history: [] });
                renderHistory();
            }
        });
    }

    // Listen for progress messages from content script or background
    chrome.runtime.onMessage.addListener((request) => {
        if (request.action === 'progress') {
            updateStatus(statusDiv, request.message);
        }
        if (request.action === 'scrape_complete') {
            handleScrapeComplete(request.data, statusDiv, previewDiv);
        }
        if (request.action === 'scrape_error') {
            updateStatus(statusDiv, 'Error: ' + request.error);
        }
    });

    if (grabBtn) {
        grabBtn.addEventListener('click', async () => {
            await handleGrabClick(statusDiv, previewDiv);
        });
    }
});

async function handleScrapeComplete(data, statusDiv, previewDiv) {
    Logger.info('Scrape successful', data);
    
    // Render preview using our "templating engine"
    if (previewDiv) {
        previewDiv.innerHTML = renderProfileTemplate(data);
    }

    // NOTE: History saving and Auto-download are now handled by the Background Script
    // to ensure they work even if the popup is closed.
    
    updateStatus(statusDiv, 'Done!');
}

/**
 * Handles the click event on the Grab button.
 * @param {HTMLElement} statusDiv - The status display element.
 * @param {HTMLElement} previewDiv - The preview display element.
 */
async function handleGrabClick(statusDiv, previewDiv) {
    try {
        const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
        
        if (!tab || !tab.url.includes('linkedin.com/in/')) {
            updateStatus(statusDiv, 'Not a LinkedIn profile page.');
            return;
        }

        updateStatus(statusDiv, 'Starting scrape...');
        if (previewDiv) previewDiv.innerHTML = ''; // Clear previous preview
        
        // Send start command to Background Script
        const response = await chrome.runtime.sendMessage({ action: 'start_scrape', tabId: tab.id });
        
        if (response && response.error) {
            updateStatus(statusDiv, 'Error: ' + response.error);
        }

    } catch (error) {
        Logger.error('Popup error', error);
        updateStatus(statusDiv, 'Error: ' + error.message);
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

    history.forEach((item, index) => {
        const div = document.createElement('div');
        div.className = 'history-item';
        div.innerHTML = `
            <div class="history-info">
                <strong>${escapeHtml(item.name)}</strong><br>
                <span class="history-date">${new Date(item.date).toLocaleString()}</span>
            </div>
            <div class="history-actions">
                <button class="secondary-btn icon-btn download-btn" title="Download JSON">⬇</button>
                <button class="secondary-btn icon-btn delete-btn" title="Delete">✖</button>
            </div>
        `;
        
        // Download Handler
        div.querySelector('.download-btn').addEventListener('click', (e) => {
            e.stopPropagation();
            downloadJSON(item.data, item.filename);
        });

        // Delete Handler
        div.querySelector('.delete-btn').addEventListener('click', async (e) => {
            e.stopPropagation();
            await deleteHistoryItem(index);
        });

        historyList.appendChild(div);
    });
}

async function deleteHistoryItem(index) {
    const result = await chrome.storage.local.get(['history']);
    const history = result.history || [];
    
    if (index >= 0 && index < history.length) {
        history.splice(index, 1);
        await chrome.storage.local.set({ history });
        renderHistory();
    }
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
