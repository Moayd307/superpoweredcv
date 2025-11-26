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
        const mainResult = await sendMessageToTab(tabId, { action: 'scrape_main' });
        // Ensure content script is ready
        if (!mainResult || !mainResult.profile) {
            throw new Error('Failed to scrape main profile');
        }onst mainResult = await sendMessageToTab(tabId, { action: 'scrape_main' });
        
        scrapingState.data = mainResult.profile;{
        scrapingState.queue = mainResult.sectionsToScrape || [];se refresh the page.');
        }
        // 2. Process Queue
        while (scrapingState.queue.length > 0) {
            const item = scrapingState.queue.shift();rape || [];
            updateStatus(`Navigating to ${item.key}...`);
            . Process Queue
            // NavigateState.queue.length > 0) {
            await chrome.tabs.update(tabId, { url: item.url });
            await waitForTabLoad(tabId);${item.key}...`);
            
            // Humanized Wait
            const delay = Math.floor(Math.random() * 2000) + 2000; // 2-4s
            updateStatus(`Reading ${item.key} (waiting ${delay}ms)...`);
            await new Promise(r => setTimeout(r, delay));
            // Humanized Wait
            // Scrape Sectionh.floor(Math.random() * 2000) + 2000; // 2-4s
            const sectionResult = await sendMessageToTab(tabId, { action: 'scrape_section', section: item.key });
            if (sectionResult && sectionResult.data) {));
                scrapingState.data[item.key] = sectionResult.data;
            }/ Scrape Section
        }   const sectionResult = await sendMessageToTab(tabId, { action: 'scrape_section', section: item.key });
            if (sectionResult && sectionResult.data) {
        // 3. Return to original URLtem.key] = sectionResult.data;
        updateStatus('Returning to profile...');
        await chrome.tabs.update(tabId, { url: scrapingState.originalUrl });
        await waitForTabLoad(tabId);
        // 3. Return to original URL
        updateStatus('Done!');g to profile...');
        scrapingState.isRunning = false;{ url: scrapingState.originalUrl });
        await waitForTabLoad(tabId);
        // Notify Popup of completion
        chrome.runtime.sendMessage({ action: 'scrape_complete', data: scrapingState.data });
        scrapingState.isRunning = false;
        return { success: true };
        // Notify Popup of completion
    } catch (error) {e.sendMessage({ action: 'scrape_complete', data: scrapingState.data });
        console.error('Scraping error:', error);
        scrapingState.isRunning = false;
        scrapingState.status = 'Error: ' + error.message;
        chrome.runtime.sendMessage({ action: 'scrape_error', error: error.message });
        return { error: error.message }; error);
    }   scrapingState.isRunning = false;
}       scrapingState.status = 'Error: ' + error.message;
        chrome.runtime.sendMessage({ action: 'scrape_error', error: error.message });
function updateStatus(msg) {r.message };
    scrapingState.status = msg;
    chrome.runtime.sendMessage({ action: 'progress', message: msg });
}
function updateStatus(msg) {
    scrapingState.status = msg;
    chrome.runtime.sendMessage({ action: 'progress', message: msg });
}

function sendMessageToTab(tabId, message) {
    return new Promise((resolve) => {
        chrome.tabs.sendMessage(tabId, message, (response) => {
            if (chrome.runtime.lastError) {
                console.warn('Tab message failed:', chrome.runtime.lastError.message);
                // Retry once after 1s if it's a connection error
                if (chrome.runtime.lastError.message.includes('Receiving end does not exist')) {
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
    // Try pinging first
    for (let i = 0; i < 3; i++) {
        const response = await sendMessageToTab(tabId, { action: 'ping' });
        if (response && response.status === 'alive') return true;
        await new Promise(r => setTimeout(r, 200));
    }

    // If ping fails, try injecting
    try {
        console.log('Content script not responding, attempting injection...');
        await chrome.scripting.executeScript({
            target: { tabId: tabId },
            files: ['src/content/index.js']
        });
        
        // Wait a bit for it to initialize
        await new Promise(r => setTimeout(r, 500));
        
        // Ping again
        const response = await sendMessageToTab(tabId, { action: 'ping' });
        return response && response.status === 'alive';
    } catch (e) {
        console.error('Injection failed:', e);
        return false;
    }
}











}    return false;    }        attempts++;        await new Promise(r => setTimeout(r, 500));        if (response && response.status === 'alive') return true;        const response = await sendMessageToTab(tabId, { action: 'ping' });    while (attempts < 5) {    let attempts = 0;async function ensureContentScriptReady(tabId) {        chrome.tabs.onUpdated.addListener(listener);
    });
}
