/**
 * @file content/index.js
 * @description Content script for scraping LinkedIn profiles.
 */

// Check if we are in a Node.js environment (for testing)
const isNode = typeof module !== 'undefined' && module.exports;

// Mock Logger if not available (e.g. if not bundled)
const Logger = {
    debug: (...args) => console.log('[SuperpoweredCV DEBUG]:', ...args),
    error: (...args) => console.error('[SuperpoweredCV ERROR]:', ...args),
    info: (...args) => console.info('[SuperpoweredCV INFO]:', ...args)
};

if (!isNode) {
    // We removed the strict idempotency check (window.hasSuperpoweredCVContentScript)
    // to allow the background script to re-inject us if the connection is lost (e.g. after extension reload).
    // The background script manages the "only inject if needed" logic via pings.
    
    window.hasSuperpoweredCVContentScript = true; // Mark as present for debugging
    
    /**
     * Listen for messages from the popup or background.
     */
    chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
            if (request.action === 'scrape_main') {
                Logger.info('Received scrape_main request');
                scrapeMainProfile()
                    .then(data => sendResponse({ profile: data.profile, sectionsToScrape: data.sectionsToScrape }))
                    .catch(error => sendResponse({ error: error.message }));
            }
            else if (request.action === 'scrape_section') {
                Logger.info('Received scrape_section request', request.section);
                scrapeSpecificSection(request.section)
                    .then(data => sendResponse({ data: data }))
                    .catch(error => sendResponse({ error: error.message }));
            }
            else if (request.action === 'ping') {
                sendResponse({ status: 'alive' });
            }
            return true; // Keep channel open for async response
    });
    Logger.info('SuperpoweredCV Content Script Initialized');
}

/**
 * Scrapes the main profile page and identifies sections needing navigation.
 */
async function scrapeMainProfile() {
    const url = window.location.href;
    const sectionsToScrape = [];
    const profile = {
        name: getText('h1'),
        headline: getText('.text-body-medium.break-words'),
        location: getText('.text-body-small.inline.t-black--light.break-words'),
        about: getAbout(),
        url: url
    };

    // Define sections and their scrapers
    const sections = [
        { key: 'experience', id: 'experience', scraper: getExperienceFromDoc },
        { key: 'education', id: 'education', scraper: getEducationFromDoc },
        { key: 'languages', id: 'languages', scraper: getLanguagesFromDoc },
        { key: 'volunteering', id: 'volunteering', scraper: getVolunteeringFromDoc },
        { key: 'skills', id: 'skills', scraper: getSkillsFromDoc },
        { key: 'projects', id: 'projects', scraper: getProjectsFromDoc },
        { key: 'courses', id: 'courses', scraper: getCoursesFromDoc },
        { key: 'publications', id: 'publications', scraper: getPublicationsFromDoc },
        { key: 'patents', id: 'patents', scraper: getPatentsFromDoc },
        { key: 'organizations', id: 'organizations', scraper: getOrganizationsFromDoc }
    ];

    for (const sec of sections) {
        const url = getSectionUrl(sec.id);
        if (url) {
            sectionsToScrape.push({ key: sec.key, url });
            profile[sec.key] = []; // Placeholder
        } else {
            profile[sec.key] = sec.scraper(document);
        }
    }

    // Contact info is special (overlay)
    profile.contactInfo = await getContactInfo();

    return { profile, sectionsToScrape };
}

/**
 * Scrapes a specific section from the current page (assumed to be a details page).
 */
async function scrapeSpecificSection(sectionKey) {
    switch (sectionKey) {
        case 'experience': return getExperienceFromDoc(document);
        case 'education': return getEducationFromDoc(document);
        case 'languages': return getLanguagesFromDoc(document);
        case 'volunteering': return getVolunteeringFromDoc(document);
        case 'skills': return getSkillsFromDoc(document);
        case 'projects': return getProjectsFromDoc(document);
        case 'courses': return getCoursesFromDoc(document);
        case 'publications': return getPublicationsFromDoc(document);
        case 'patents': return getPatentsFromDoc(document);
        case 'organizations': return getOrganizationsFromDoc(document);
        default: return [];
    }
}

/**
 * Helper to get the "Show all" URL for a section.
 */
function getSectionUrl(sectionId) {
    const footerLink = document.querySelector(`#${sectionId}`)?.closest('.artdeco-card')?.querySelector('div.pvs-list__footer-wrapper a');
    return footerLink ? footerLink.href : null;
}



/**
 * Helper to get text content from a selector.
 * @param {string} selector - CSS selector.
 * @returns {string} The trimmed text content or empty string.
 */
function getText(selector) {
    if (typeof document === 'undefined') return '';
    const el = document.querySelector(selector);
    return el ? el.textContent.trim() : '';
}

/**
 * Helper to extract description text, handling "show more" expansion.
 * @param {Element} element - The element containing the description.
 * @returns {string} The description text.
 */
function getDescription(element) {
    // Try to find the container with any class starting with inline-show-more-text
    const descriptionEl = element.querySelector('[class*="inline-show-more-text"]');
    if (descriptionEl) {
        const visibleSpan = descriptionEl.querySelector('span[aria-hidden="true"]');
        if (visibleSpan) return visibleSpan.textContent.trim();
        
        // Fallback: clone and remove visually-hidden
        const clone = descriptionEl.cloneNode(true);
        const hidden = clone.querySelectorAll('.visually-hidden');
        hidden.forEach(el => el.remove());
        return clone.textContent.trim();
    }
    return '';
}

/**
 * Helper to fetch a URL and return the document.
 * @param {string} url 
 * @returns {Promise<Document>}
 */
async function fetchDocument(url) {
    try {
        const response = await fetch(url);
        const text = await response.text();
        const parser = new DOMParser();
        return parser.parseFromString(text, 'text/html');
    } catch (e) {
        Logger.error(`Failed to fetch ${url}`, e);
        return null;
    }
}

/**
 * Scrapes the 'About' section.
 * @returns {string} The about text.
 */
function getAbout() {
    if (typeof document === 'undefined') return '';
    const section = document.getElementById('about');
    if (section) {
        const parent = section.closest('.artdeco-card');
        if (parent) {
            return getDescription(parent);
        }
    }
    return '';
}

/**
 * Helper to extract experience items from a document.
 * @param {Document} doc - The document to scrape (main page or details page).
 * @returns {Array<Object>} List of experience items.
 */
function getExperienceFromDoc(doc) {
    const items = getSectionItems(doc, 'experience');
    const experiences = [];

    items.forEach(item => {
        // Generic extraction strategy:
        // 1. Get all visible spans (aria-hidden="true")
        // 2. Filter out empty/separator spans
        // 3. Map by position (Title, Company, Date, Location)
        
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.textContent.trim()).filter(t => t && t !== '·');

        if (texts.length < 2) return; // Need at least Title and Company

        let title = texts[0];
        let company = texts[1];
        let dateRange = '';
        let tenure = '';
        let location = '';

        // Try to identify date/tenure by pattern
        // Date usually contains year (19xx or 20xx) or month names
        // Tenure usually contains "yr" or "mos"
        
        // Find index of date-like string
        const dateIndex = texts.findIndex(t => /\d{4}|Present/i.test(t) && (t.includes(' - ') || t.includes('·')));
        
        if (dateIndex > 1) {
            // If we found a date later, everything before it might be title/company parts
            // But usually Title is 0, Company is 1.
            const dateText = texts[dateIndex];
            const parts = dateText.split('·').map(s => s.trim());
            dateRange = parts[0] || '';
            if (parts.length > 1) tenure = parts[1];
            
            if (texts[dateIndex + 1]) location = texts[dateIndex + 1];
        } else if (texts[2]) {
             // Fallback: assume 3rd item is date
             const parts = texts[2].split('·').map(s => s.trim());
             dateRange = parts[0];
             if (parts.length > 1) tenure = parts[1];
             if (texts[3]) location = texts[3];
        }

        // Description
        const description = getDescription(item);

        // Skills
        let skills = [];
        const skillsContainer = Array.from(item.querySelectorAll('div')).find(div => div.textContent.includes('Skills:'));
        if (skillsContainer) {
            // Be careful not to grab the whole item text
            // Usually skills are in a distinct div
            const fullText = skillsContainer.textContent.trim();
            // Check if this div is actually small enough to be just skills
            if (fullText.length < 500) {
                 const skillsText = fullText.replace(/^Skills:\s*/i, '');
                 skills = skillsText.split('·').map(s => s.trim());
            }
        }

        experiences.push({ title, company, date_range: dateRange, tenure, location, description, skills });
    });

    return experiences;
}

/**
 * Helper to extract volunteering items from a document.
 * @param {Document} doc - The document to scrape.
 * @returns {Array<Object>} List of volunteering items.
 */
function getVolunteeringFromDoc(doc) {
    const items = getSectionItems(doc, 'volunteering');
    return items.map(item => {
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.textContent.trim()).filter(t => t && t !== '·');

        let role = texts[0] || '';
        let organization = texts[1] || '';
        let dateRange = '';
        let tenure = '';
        let description = '';

        // Date logic
        const dateIndex = texts.findIndex(t => /\d{4}|Present/i.test(t));
        if (dateIndex > 1) {
            const parts = texts[dateIndex].split('·').map(s => s.trim());
            dateRange = parts[0];
            if (parts.length > 1) tenure = parts[1];
        }

        // Description
        const descriptionEl = item.querySelector('.inline-show-more-text');
        if (descriptionEl) {
            const visibleSpan = descriptionEl.querySelector('span[aria-hidden="true"]');
            description = visibleSpan ? visibleSpan.innerText.trim() : descriptionEl.innerText.trim();
        }

        return { role, organization, date_range: dateRange, tenure, description };
    }).filter(i => i.role);
}



/**
 * Helper to extract languages from a document.
 * @param {Document} doc - The document to scrape.
 * @returns {Array<Object>} List of languages.
 */
function getLanguagesFromDoc(doc) {
    const items = getSectionItems(doc, 'languages');
    return items.map(item => {
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.innerText.trim()).filter(t => t && t !== '·');
        
        return {
            name: texts[0] || '',
            proficiency: texts[1] || ''
        };
    }).filter(i => i.name);
}

/**
 * Helper to extract education items from a document.
 * @param {Document} doc - The document to scrape.
 * @returns {Array<Object>} List of education items.
 */
function getEducationFromDoc(doc) {
    const items = getSectionItems(doc, 'education');
    return items.map(item => {
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.textContent.trim()).filter(t => t && t !== '·');

        let school = texts[0] || '';
        let degree = texts[1] || '';
        let dateRange = '';
        let description = '';

        // Try to find date
        const dateIndex = texts.findIndex(t => /\d{4}/.test(t));
        if (dateIndex > 1) {
            dateRange = texts[dateIndex];
        }

        // Description
        const descriptionEl = item.querySelector('.inline-show-more-text');
        if (descriptionEl) {
            const visibleSpan = descriptionEl.querySelector('span[aria-hidden="true"]');
            description = visibleSpan ? visibleSpan.innerText.trim() : descriptionEl.innerText.trim();
        }

        return { school, degree, date_range: dateRange, description };
    }).filter(i => i.school);
}

// ...existing code...
// Removed unused getSkills
// ...existing code...

/**
 * Helper to extract skills from a document.
 * @param {Document} doc - The document to scrape.
 * @returns {Array<string>} List of skills.
 */
function getSkillsFromDoc(doc) {
    const items = getSectionItems(doc, 'skills');
    return items.map(item => {
        const skillEl = item.querySelector('.display-flex.align-items-center.mr1.hoverable-link-text span[aria-hidden="true"]') || item.querySelector('span[aria-hidden="true"]');
        return skillEl ? skillEl.textContent.trim() : '';
    }).filter(s => s);
}

/**
 * Helper to extract projects from a document.
 * @param {Document} doc - The document to scrape.
 * @returns {Array<Object>} List of projects.
 */
function getProjectsFromDoc(doc) {
    const items = getSectionItems(doc, 'projects');
    return items.map(item => {
        // Generic extraction
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.textContent.trim()).filter(t => t && t !== '·');
        
        let title = texts[0] || '';
        let date = '';
        let link = '';

        // Try to find date
        const dateIndex = texts.findIndex(t => /\d{4}|Present/i.test(t));
        if (dateIndex > 0) date = texts[dateIndex];

        // Description
        const description = getDescription(item);

        // Link
        const linkEl = item.querySelector('a.optional-action-target-wrapper');
        if (linkEl) link = linkEl.href;

        return { title, date, description, link };
    }).filter(i => i.title);
}

function getPublicationsFromDoc(doc) {
    const items = getSectionItems(doc, 'publications');
    return items.map(item => {
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.innerText.trim()).filter(t => t && t !== '·');

        let title = texts[0] || '';
        let date = '';
        let link = '';

        // Date logic
        const dateIndex = texts.findIndex(t => /\d{4}/.test(t));
        if (dateIndex > 0) date = texts[dateIndex];

        // Description
        const description = getDescription(item);

        // Link
        const linkEl = item.querySelector('a.optional-action-target-wrapper');
        if (linkEl) link = linkEl.href;

        return { title, date, description, link };
    }).filter(i => i.title);
}

function getCoursesFromDoc(doc) {
    const items = getSectionItems(doc, 'courses');
    return items.map(item => {
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.innerText.trim()).filter(t => t && t !== '·');
        
        return {
            name: texts[0] || '',
            number: texts[1] || ''
        };
    }).filter(i => i.name);
}

function getPatentsFromDoc(doc) {
    const items = getSectionItems(doc, 'patents');
    return items.map(item => {
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.innerText.trim()).filter(t => t && t !== '·');

        let title = texts[0] || '';
        let number = texts[1] || '';
        
        const description = getDescription(item);
        
        return { title, number, description };
    }).filter(i => i.title);
}

function getOrganizationsFromDoc(doc) {
    const items = getSectionItems(doc, 'organizations');
    return items.map(item => {
        const spans = Array.from(item.querySelectorAll('span[aria-hidden="true"]'));
        const texts = spans.map(s => s.textContent.trim()).filter(t => t && t !== '·');

        let name = texts[0] || '';
        let role = texts[1] || '';
        let date = '';

        const dateIndex = texts.findIndex(t => /\d{4}/.test(t));
        if (dateIndex > 1) date = texts[dateIndex];

        const description = getDescription(item);

        return { name, role, date, description };
    }).filter(i => i.name);
}

/**
 * Scrapes Contact Info from the overlay.
 * @returns {Promise<Object>} Contact info object.
 */
async function getContactInfo() {
    if (typeof window === 'undefined') return {};
    
    // Construct overlay URL
    const match = window.location.href.match(new RegExp('/in/([^/]+)'));
    if (!match) return {};
    
    const slug = match[1];
    const url = `https://www.linkedin.com/in/${slug}/overlay/contact-info/`;
    
    const doc = await fetchDocument(url);
    if (!doc) return {};

    const contactInfo = {};
    
    // Generic section parser for contact info
    // Look for sections by header text
    const sections = Array.from(doc.querySelectorAll('section'));
    
    sections.forEach(section => {
        const header = section.querySelector('h3');
        if (!header) return;
        const headerText = header.textContent.toLowerCase();
        
        if (headerText.includes('email')) {
            const link = section.querySelector('a');
            if (link) contactInfo.email = link.textContent.trim();
        } else if (headerText.includes('phone')) {
            const items = section.querySelectorAll('li, span.t-14');
            contactInfo.phone = Array.from(items).map(i => i.textContent.trim()).filter(t => t);
        } else if (headerText.includes('website')) {
            const items = section.querySelectorAll('a');
            contactInfo.websites = Array.from(items).map(i => i.href);
        } else if (headerText.includes('twitter') || headerText.includes('x')) { // X (Twitter)
            const link = section.querySelector('a');
            if (link) contactInfo.twitter = link.href;
        }
    });

    return contactInfo;
}

/**
 * Helper to get items from a section, handling both main page and details page structures.
 * @param {Document} doc 
 * @param {string} sectionId 
 * @returns {Array<Element>}
 */
function getSectionItems(doc, sectionId) {
    let items = [];
    
    // If we are on the main page, find the section by ID
    if (doc === document) {
        const section = doc.getElementById(sectionId);
        if (section) {
            const parent = section.closest('.artdeco-card');
            if (parent) {
                items = parent.querySelectorAll('li.artdeco-list__item');
            }
        }
    } else {
        // If we are on a details page (fetched doc), the structure is usually a pvs-list
        items = doc.querySelectorAll('.pvs-list__paged-list-item, li.artdeco-list__item');
    }
    
    return Array.from(items);
}

// Export for testing
if (isNode) {
    module.exports = {
        scrapeMainProfile,
        getText,
        getAbout,
        getExperienceFromDoc,
        getEducationFromDoc,
        getSkillsFromDoc
    };
}