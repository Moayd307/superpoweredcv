const content = require('../src/content/index.js');

describe('Content Script', () => {
    afterEach(() => {
        jest.restoreAllMocks();
    });

    test('getText returns empty string if document is undefined', () => {
        // We are in jsdom environment, so document exists.
        // We can mock querySelector to return null
        jest.spyOn(document, 'querySelector').mockReturnValue(null);
        expect(content.getText('h1')).toBe('');
    });

    test('getText returns text content', () => {
        const dummyEl = document.createElement('div');
        dummyEl.textContent = '  Hello World  ';
        jest.spyOn(document, 'querySelector').mockReturnValue(dummyEl);
        expect(content.getText('h1')).toBe('Hello World');
    });

    test('getAbout extracts description', () => {
        document.body.innerHTML = `
            <div class="artdeco-card">
                <div id="about"></div>
                <div class="inline-show-more-text">
                    <span aria-hidden="true">About Me Text</span>
                </div>
            </div>
        `;
        expect(content.getAbout()).toBe('About Me Text');
    });

    test('getExperienceFromDoc extracts items', () => {
        document.body.innerHTML = `
            <div id="experience">
                <div class="pvs-list__paged-list-item">
                    <span aria-hidden="true">Software Engineer</span>
                    <span aria-hidden="true">Tech Corp</span>
                    <span aria-hidden="true">2020 - Present · 3 yrs</span>
                    <span aria-hidden="true">New York</span>
                    <div class="inline-show-more-text">
                        <span aria-hidden="true">Built cool stuff.</span>
                    </div>
                </div>
            </div>
        `;
        // We need to mock getSectionItems or ensure the DOM structure matches what getSectionItems expects.
        // getSectionItems looks for #experience -> closest card -> li.artdeco-list__item OR .pvs-list__paged-list-item
        // Since we are passing 'doc' as document, it uses the first branch (main page).
        // But getExperienceFromDoc calls getSectionItems(doc, 'experience').
        
        // Let's mock getSectionItems behavior by constructing the DOM correctly for the "details page" path 
        // or just mocking the function if we could, but it's internal.
        // Let's construct a DOM that matches the "details page" selector since we can pass a mock doc.
        
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">Software Engineer</span>
                <span aria-hidden="true">Tech Corp</span>
                <span aria-hidden="true">2020 - Present · 3 yrs</span>
                <span aria-hidden="true">New York</span>
                <div class="inline-show-more-text">
                    <span aria-hidden="true">Built cool stuff.</span>
                </div>
            </div>
        `;
        
        const results = content.getExperienceFromDoc(mockDoc);
        expect(results).toHaveLength(1);
        expect(results[0].title).toBe('Software Engineer');
        expect(results[0].company).toBe('Tech Corp');
        expect(results[0].description).toBe('Built cool stuff.');
    });

    test('getEducationFromDoc extracts items', () => {
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">University of Tech</span>
                <span aria-hidden="true">Bachelor of Science</span>
                <span aria-hidden="true">2016 - 2020</span>
                <div class="inline-show-more-text">
                    <span aria-hidden="true">Graduated with honors.</span>
                </div>
            </div>
        `;
        
        const results = content.getEducationFromDoc(mockDoc);
        expect(results).toHaveLength(1);
        expect(results[0].school).toBe('University of Tech');
        expect(results[0].degree).toBe('Bachelor of Science');
        expect(results[0].date_range).toBe('2016 - 2020');
        expect(results[0].description).toBe('Graduated with honors.');
    });

    test('getSkillsFromDoc extracts skills', () => {
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">JavaScript</span>
            </div>
            <div class="pvs-list__paged-list-item">
                <div class="display-flex align-items-center mr1 hoverable-link-text">
                    <span aria-hidden="true">Rust</span>
                </div>
            </div>
        `;
        
        const results = content.getSkillsFromDoc(mockDoc);
        expect(results).toEqual(['JavaScript', 'Rust']);
    });

    test('getSkillsFromDoc extracts skills using data-field selector', () => {
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div data-field="skill_page_skill_topic">
                <span aria-hidden="true">Python</span>
            </div>
            <div data-field="skill_page_skill_topic">
                <span aria-hidden="true">Machine Learning</span>
            </div>
        `;
        
        const results = content.getSkillsFromDoc(mockDoc);
        expect(results).toEqual(['Python', 'Machine Learning']);
    });

    test('getLanguagesFromDoc extracts languages', () => {
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">English</span>
                <span aria-hidden="true">Native or bilingual proficiency</span>
            </div>
        `;
        
        const results = content.getLanguagesFromDoc(mockDoc);
        expect(results).toHaveLength(1);
        expect(results[0].name).toBe('English');
        expect(results[0].proficiency).toBe('Native or bilingual proficiency');
    });

    test('getVolunteeringFromDoc extracts volunteering', () => {
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">Volunteer Role</span>
                <span aria-hidden="true">NonProfit Org</span>
                <span aria-hidden="true">Jan 2020 - Present · 3 yrs</span>
                <div class="inline-show-more-text">
                    <span aria-hidden="true">Helped people.</span>
                </div>
            </div>
        `;
        
        const results = content.getVolunteeringFromDoc(mockDoc);
        expect(results).toHaveLength(1);
        expect(results[0].role).toBe('Volunteer Role');
        expect(results[0].organization).toBe('NonProfit Org');
        expect(results[0].date_range).toBe('Jan 2020 - Present');
        expect(results[0].tenure).toBe('3 yrs');
        expect(results[0].description).toBe('Helped people.');
    });

    test('getProjectsFromDoc extracts projects', () => {
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">Super Project</span>
                <span aria-hidden="true">Jan 2021 - Dec 2021</span>
                <a class="optional-action-target-wrapper" href="https://example.com/project"></a>
                <div class="inline-show-more-text">
                    <span aria-hidden="true">A great project.</span>
                </div>
            </div>
        `;
        
        const results = content.getProjectsFromDoc(mockDoc);
        expect(results).toHaveLength(1);
        expect(results[0].title).toBe('Super Project');
        expect(results[0].date).toBe('Jan 2021 - Dec 2021');
        expect(results[0].link).toBe('https://example.com/project');
        expect(results[0].description).toBe('A great project.');
    });

    test('getPublicationsFromDoc extracts publications', () => {
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">Research Paper</span>
                <span aria-hidden="true">Published in Journal</span>
                <span aria-hidden="true">2022</span>
                <a class="optional-action-target-wrapper" href="https://example.com/paper"></a>
                <div class="inline-show-more-text">
                    <span aria-hidden="true">Abstract of the paper.</span>
                </div>
            </div>
        `;
        
        const results = content.getPublicationsFromDoc(mockDoc);
        expect(results).toHaveLength(1);
        expect(results[0].title).toBe('Research Paper');
        expect(results[0].date).toBe('2022');
        expect(results[0].link).toBe('https://example.com/paper');
        expect(results[0].description).toBe('Abstract of the paper.');
    });

    test('getCoursesFromDoc extracts courses', () => {
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">Advanced Algorithms</span>
                <span aria-hidden="true">CS101</span>
            </div>
        `;
        
        const results = content.getCoursesFromDoc(mockDoc);
        expect(results).toHaveLength(1);
        expect(results[0].name).toBe('Advanced Algorithms');
        expect(results[0].number).toBe('CS101');
    });

    test('getPatentsFromDoc extracts patents', () => {
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">New Invention</span>
                <span aria-hidden="true">US123456</span>
                <div class="inline-show-more-text">
                    <span aria-hidden="true">Patent description.</span>
                </div>
            </div>
        `;
        
        const results = content.getPatentsFromDoc(mockDoc);
        expect(results).toHaveLength(1);
        expect(results[0].title).toBe('New Invention');
        expect(results[0].number).toBe('US123456');
        expect(results[0].description).toBe('Patent description.');
    });

    test('getOrganizationsFromDoc extracts organizations', () => {
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">Tech Club</span>
                <span aria-hidden="true">Member</span>
                <span aria-hidden="true">2019 - 2020</span>
                <div class="inline-show-more-text">
                    <span aria-hidden="true">Participated in events.</span>
                </div>
            </div>
        `;
        
        const results = content.getOrganizationsFromDoc(mockDoc);
        expect(results).toHaveLength(1);
        expect(results[0].name).toBe('Tech Club');
        expect(results[0].role).toBe('Member');
        expect(results[0].date).toBe('2019 - 2020');
        expect(results[0].description).toBe('Participated in events.');
    });

    test('getInterestsFromDoc extracts interests', () => {
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">Technology</span>
            </div>
        `;
        
        const results = content.getInterestsFromDoc(mockDoc);
        expect(results).toHaveLength(1);
        expect(results[0].name).toBe('Technology');
    });

    test('getAccomplishmentsFromDoc extracts accomplishments', () => {
        const mockDoc = document.implementation.createHTMLDocument();
        mockDoc.body.innerHTML = `
            <div class="pvs-list__paged-list-item">
                <span aria-hidden="true">Best Coder Award</span>
            </div>
        `;
        
        const results = content.getAccomplishmentsFromDoc(mockDoc);
        expect(results).toHaveLength(1);
        expect(results[0].title).toBe('Best Coder Award');
    });

    test('isOpenToWork returns true if frame is present', () => {
        document.body.innerHTML = '';
        const div = document.createElement('div');
        div.className = 'pv-top-card-profile-picture';
        const img = document.createElement('img');
        img.setAttribute('title', 'John Doe #OPEN_TO_WORK');
        div.appendChild(img);
        document.body.appendChild(div);
        
        expect(content.isOpenToWork()).toBe(true);
    });
});
