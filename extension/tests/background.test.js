// Mock chrome API BEFORE requiring the background script
global.chrome = {
    runtime: {
        onMessage: {
            addListener: jest.fn()
        },
        sendMessage: jest.fn(),
        lastError: null
    },
    tabs: {
        get: jest.fn(),
        update: jest.fn(),
        sendMessage: jest.fn(),
        onUpdated: {
            addListener: jest.fn(),
            removeListener: jest.fn()
        }
    },
    scripting: {
        executeScript: jest.fn()
    }
};

const { handleStartScrape, ensureContentScriptReady, sendMessageToTab } = require('../src/background/index.js');

describe('Background Script', () => {
    beforeEach(() => {
        jest.clearAllMocks();
    });

    test('ensureContentScriptReady returns true if ping succeeds', async () => {
        chrome.tabs.sendMessage.mockImplementation((tabId, msg, cb) => {
            cb({ status: 'alive' });
        });

        const result = await ensureContentScriptReady(123);
        expect(result).toBe(true);
        expect(chrome.tabs.sendMessage).toHaveBeenCalled();
    });

    test('ensureContentScriptReady injects script if ping fails', async () => {
        // First ping fails (no response or error)
        chrome.tabs.sendMessage.mockImplementationOnce((tabId, msg, cb) => {
            chrome.runtime.lastError = { message: 'Receiving end does not exist' };
            cb(null);
        });
        // Subsequent pings (after injection) succeed
        chrome.tabs.sendMessage.mockImplementation((tabId, msg, cb) => {
            chrome.runtime.lastError = null;
            cb({ status: 'alive' });
        });

        const result = await ensureContentScriptReady(123);
        
        expect(chrome.scripting.executeScript).toHaveBeenCalledWith({
            target: { tabId: 123 },
            files: ['src/content/index.js']
        });
        expect(result).toBe(true);
    });
});
