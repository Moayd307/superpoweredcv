/**
 * Event listener for the "Grab Profile" button.
 * Initiates the scraping process by sending a message to the active tab.
 */
document.getElementById('grabBtn').addEventListener('click', async () => {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  
  if (!tab.url.includes('linkedin.com/in/')) {
    document.getElementById('status').textContent = 'Not a LinkedIn profile page.';
    return;
  }

  document.getElementById('status').textContent = 'Scraping...';

  try {
    const response = await chrome.tabs.sendMessage(tab.id, { action: 'scrape' });
    if (response && response.data) {
      downloadJSON(response.data, `profile_${response.data.name || 'unknown'}.json`);
      document.getElementById('status').textContent = 'Done!';
    } else {
      document.getElementById('status').textContent = 'Failed to scrape.';
    }
  } catch (error) {
    console.error(error);
    document.getElementById('status').textContent = 'Error: ' + error.message;
  }
});

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
