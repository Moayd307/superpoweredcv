/**
 * Listen for messages from the popup.
 */
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === 'scrape') {
    const data = scrapeProfile();
    sendResponse({ data: data });
  }
});

/**
 * Scrapes the LinkedIn profile data from the current page.
 * @returns {Object} The scraped profile object.
 */
function scrapeProfile() {
  const profile = {
    name: getText('h1'),
    headline: getText('.text-body-medium.break-words'),
    location: getText('.text-body-small.inline.t-black--light.break-words'),
    about: getAbout(),
    experience: getExperience(),
    education: getEducation(),
    skills: getSkills(),
    url: window.location.href
  };
  return profile;
}

/**
 * Helper to get text content from a selector.
 * @param {string} selector - CSS selector.
 * @returns {string} The trimmed text content or empty string.
 */
function getText(selector) {
  const el = document.querySelector(selector);
  return el ? el.innerText.trim() : '';
}

/**
 * Scrapes the 'About' section.
 * @returns {string} The about text.
 */
function getAbout() {
  // LinkedIn often puts About in a section with id="about"
  // But the text might be in a span or div inside
  const section = document.getElementById('about');
  if (section) {
    // Look for the next sibling div that contains the text
    // This is tricky as DOM structure varies.
    // Try to find the nearest text container.
    const textContainer = section.parentElement.querySelector('.display-flex.ph5.pv3 .inline-show-more-text--is-collapsed, .display-flex.ph5.pv3 .inline-show-more-text--is-expanded');
    if (textContainer) return textContainer.innerText.trim();
    
    // Fallback: try to find any text block in the parent container of the anchor
    const parent = section.closest('.artdeco-card');
    if (parent) {
        const text = parent.querySelector('.inline-show-more-text span[aria-hidden="true"]');
        if (text) return text.innerText.trim();
    }
  }
  return '';
}

/**
 * Scrapes the 'Experience' section.
 * @returns {Array<Object>} List of experience items.
 */
function getExperience() {
  const experiences = [];
  const section = document.getElementById('experience');
  if (section) {
    const parent = section.closest('.artdeco-card');
    if (parent) {
      const items = parent.querySelectorAll('li.artdeco-list__item');
      items.forEach(item => {
        const titleEl = item.querySelector('.display-flex.align-items-center.mr1.t-bold span[aria-hidden="true"]');
        const companyEl = item.querySelector('span.t-14.t-normal span[aria-hidden="true"]');
        // Date and location are often in spans with t-black--light
        const metaEls = item.querySelectorAll('span.t-14.t-normal.t-black--light span[aria-hidden="true"]');
        
        if (titleEl) {
            experiences.push({
                title: titleEl.innerText.trim(),
                company: companyEl ? companyEl.innerText.trim() : '',
                date_range: metaEls[0] ? metaEls[0].innerText.trim() : '',
                location: metaEls[1] ? metaEls[1].innerText.trim() : ''
            });
        }
      });
    }
  }
  return experiences;
}

/**
 * Scrapes the 'Education' section.
 * @returns {Array<Object>} List of education items.
 */
function getEducation() {
  const education = [];
  const section = document.getElementById('education');
  if (section) {
    const parent = section.closest('.artdeco-card');
    if (parent) {
      const items = parent.querySelectorAll('li.artdeco-list__item');
      items.forEach(item => {
        const schoolEl = item.querySelector('.display-flex.align-items-center.mr1.hoverable-link-text span[aria-hidden="true"]');
        const degreeEl = item.querySelector('span.t-14.t-normal span[aria-hidden="true"]');
        
        if (schoolEl) {
            education.push({
                school: schoolEl.innerText.trim(),
                degree: degreeEl ? degreeEl.innerText.trim() : ''
            });
        }
      });
    }
  }
  return education;
}

/**
 * Scrapes the 'Skills' section.
 * @returns {Array<string>} List of skills.
 */
function getSkills() {
  const skills = [];
  const section = document.getElementById('skills');
  if (section) {
    const parent = section.closest('.artdeco-card');
    if (parent) {
        // Usually skills are in a list, sometimes with "Show all skills" button
        // We grab the visible ones
        const items = parent.querySelectorAll('.display-flex.align-items-center.mr1.hoverable-link-text span[aria-hidden="true"]');
        items.forEach(item => {
            skills.push(item.innerText.trim());
        });
    }
  }
  return skills;
}
