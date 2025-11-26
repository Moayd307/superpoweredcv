/**
 * @file background/index.js
 * @description Background service worker to orchestrate the scraping process with navigation.
 */

let scrapingState = {
    isRunning: false,
    tabId: null,
    data: {},
    queue: [],
    originalUrl: null
};

// Listen for messages from Popup
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    if (request.action === 'start_scrape') {
        handleStartScrape(request.tabId).then(sendResponse);
        return true; // Async response
    }
    if (request.action === 'get_status') {
        sendResponse(scrapingState);
    }
});

async function handleStartScrape(tabId) {
    if (scrapingState.isRunning) return { error: 'Scraping already in progress' };
    
    try {
        scrapingState = {
            isRunning: true,
            tabId: tabId,
            data: {},
            queue: [],
            originalUrl: null,
            status: 'Starting...'
        };

        // Get current URL to return to later
        const tab = await chrome.tabs.get(tabId);
        scrapingState.originalUrl = tab.url;

        // 1. Scrape Main Page
        updateStatus('Scraping main profile...');
        
        // Ensure content script is ready
        const ready = await ensureContentScriptReady(tabId);
        if (!ready) {
            throw new Error('Could not connect to content script. Please refresh the page.');
        }

        const mainResult = await sendMessageToTab(tabId, { action: 'scrape_main' });
        
        if (!mainResult || !mainResult.profile) {
            throw new Error('Failed to scrape main profile');
        }

        scrapingState.data = mainResult.profile;
        scrapingState.queue = mainResult.sectionsToScrape || [];

        // 2. Process Queue
        while (scrapingState.queue.length > 0) {
            const item = scrapingState.queue.shift();
            updateStatus(`Navigating to ${item.key}...`);
            
            // Navigate
            await chrome.tabs.update(tabId, { url: item.url });
            await waitForTabLoad(tabId);
            
            // Humanized Wait
            const delay = Math.floor(Math.random() * 2000) + 2000; // 2-4s
            updateStatus(`Reading ${item.key} (waiting ${delay}ms)...`);
            await new Promise(r => setTimeout(r, delay));

            // Ensure script is ready after navigation
            const sectionReady = await ensureContentScriptReady(tabId);
            if (!sectionReady) {
                console.warn(`Skipping ${item.key}: Content script not ready`);
                continue;
            }

            // Scrape Section
            const sectionResult = await sendMessageToTab(tabId, { action: 'scrape_section', section: item.key });
            if (sectionResult && sectionResult.data) {
                scrapingState.data[item.key] = sectionResult.data;
            }
        }

        // 3. Return to original URL
        updateStatus('Returning to profile...');
        await chrome.tabs.update(tabId, { url: scrapingState.originalUrl });
        await waitForTabLoad(tabId);

        updateStatus('Done!');
        scrapingState.isRunning = false;
        
        // Notify Popup of completion
        chrome.runtime.sendMessage({ action: 'scrape_complete', data: scrapingState.data });
        
        // Handle persistence and download in background (works even if popup is closed)
        await handlePostScrape(scrapingState.data);
        
        return { success: true };

    } catch (error) {
        console.error('Scraping error:', error);
        scrapingState.isRunning = false;
        scrapingState.status = 'Error: ' + error.message;
        chrome.runtime.sendMessage({ action: 'scrape_error', error: error.message });
        return { error: error.message };
    }
}

function updateStatus(msg) {
    scrapingState.status = msg;
    chrome.runtime.sendMessage({ action: 'progress', message: msg });
}

/**
 * Sends a message to the tab.
 * @param {number} tabId 
 * @param {object} message 
 * @param {boolean} [retry=true] - Whether to retry on failure
 */
function sendMessageToTab(tabId, message, retry = true) {
    return new Promise((resolve) => {
        chrome.tabs.sendMessage(tabId, message, (response) => {
            if (chrome.runtime.lastError) {
                const err = chrome.runtime.lastError.message;
                // Only log if we are not just pinging or if we are out of retries
                if (retry) {
                    console.warn(`Tab message failed (${err}), retrying...`);
                    setTimeout(() => {
                        chrome.tabs.sendMessage(tabId, message, (retryResponse) => {
                            if (chrome.runtime.lastError) {
                                console.error('Retry failed:', chrome.runtime.lastError.message);
                                resolve(null);
                            } else {
                                resolve(retryResponse);
                            }
                        });
                    }, 1000);
                } else {
                    // Silent failure for pinging
                    resolve(null);
                }
            } else {
                resolve(response);
            }
        });
    });
}

function waitForTabLoad(tabId) {
    return new Promise((resolve) => {
        const listener = (tid, changeInfo) => {
            if (tid === tabId && changeInfo.status === 'complete') {
                chrome.tabs.onUpdated.removeListener(listener);
                resolve();
            }
        };
        chrome.tabs.onUpdated.addListener(listener);
    });
}

async function ensureContentScriptReady(tabId) {
    const MAX_ATTEMPTS = 10;
    
    // 1. Try pinging multiple times
    for (let i = 0; i < MAX_ATTEMPTS; i++) {
        const response = await sendMessageToTab(tabId, { action: 'ping' }, false); // No retry, silent fail
        if (response && response.status === 'alive') return true;
        await new Promise(r => setTimeout(r, 200));
    }

    // 2. If ping fails, try injecting
    try {
        console.log('Content script not responding, attempting injection...');
        await chrome.scripting.executeScript({
            target: { tabId: tabId },
            files: ['src/content/index.js']
        });
        
        // Wait for initialization
        await new Promise(r => setTimeout(r, 500));
        
        // 3. Ping again
        for (let i = 0; i < 5; i++) {
            const response = await sendMessageToTab(tabId, { action: 'ping' }, false);
            if (response && response.status === 'alive') return true;
            await new Promise(r => setTimeout(r, 200));
        }
    } catch (e) {
        console.error('Injection failed:', e);
    }
    
    return false;
}

async function handlePostScrape(data) {
    const filename = generateFilename(data.name);
    
    // Save to history
    await saveToHistory(data, filename);

    // Check settings and download
    const settings = await chrome.storage.sync.get(['autoDownload']);
    if (settings.autoDownload !== false) {
        await downloadData(data, filename);
    }
}

function generateFilename(name) {
    const cleanName = (name || 'unknown').toLowerCase().replace(/\s+/g, '');
    const now = new Date();
    const dateStr = now.toISOString().split('T')[0];
    const timeStr = now.toTimeString().split(' ')[0].replace(/:/g, '-');
    return `superpoweredcv-${cleanName}-${dateStr}-${timeStr}.json`;
}

async function saveToHistory(profile, filename) {
    const result = await chrome.storage.local.get(['history']);
    const history = result.history || [];
    const entry = {
        name: profile.name,
        headline: profile.headline,
        date: new Date().toISOString(),
        filename: filename,
        data: profile
    };
    history.unshift(entry);
    if (history.length > 50) history.pop();
    await chrome.storage.local.set({ history });
}

async function downloadData(data, filename) {
    // Use base64 to handle unicode characters correctly
    const jsonStr = JSON.stringify(data, null, 2);
    const base64 = btoa(unescape(encodeURIComponent(jsonStr)));
    const dataUrl = 'data:application/json;base64,' + base64;
    
    try {
        await chrome.downloads.download({
            url: dataUrl,
            filename: filename,
            saveAs: false
        });
    } catch (e) {
        console.error('Download failed:', e);
    }
}

// Export for testing
if (typeof module !== 'undefined') {
    module.exports = {
        handleStartScrape,
        ensureContentScriptReady,
        sendMessageToTab
    };
}
