document.addEventListener('DOMContentLoaded', function() {
    // Mobile menu toggle
    const mobileMenuToggle = document.querySelector('.mobile-menu-toggle');
    const sidebar = document.querySelector('.docs-sidebar');
    
    if (mobileMenuToggle && sidebar) {
        mobileMenuToggle.addEventListener('click', function() {
            sidebar.classList.toggle('open');
            mobileMenuToggle.innerHTML = sidebar.classList.contains('open') 
                ? '<i class="fas fa-times"></i>' 
                : '<i class="fas fa-bars"></i>';
        });
    }
    
    // Generate table of contents
    const content = document.querySelector('.docs-content');
    const toc = document.querySelector('.docs-toc-list');
    
    if (content && toc) {
        const headings = content.querySelectorAll('h2, h3');
        
        headings.forEach(heading => {
            // Create ID if not exists
            if (!heading.id) {
                heading.id = heading.textContent
                    .toLowerCase()
                    .replace(/[^\w\s-]/g, '') // Remove special characters
                    .replace(/\s+/g, '-'); // Replace spaces with hyphens
            }
            
            const item = document.createElement('li');
            const link = document.createElement('a');
            
            link.href = `#${heading.id}`;
            link.textContent = heading.textContent;
            
            if (heading.tagName === 'H3') {
                // Check if there's a previous H3 and if it doesn't have a sublist yet
                const parentList = item.closest('ul');
                const prevItem = item.previousElementSibling;
                
                if (prevItem && prevItem.querySelector('h2')) {
                    // Create a sublist
                    const sublist = document.createElement('ul');
                    sublist.appendChild(item);
                    prevItem.appendChild(sublist);
                } else {
                    item.style.paddingLeft = '1rem';
                }
            }
            
            item.appendChild(link);
            toc.appendChild(item);
        });
    }
    
    // Highlight active TOC item based on scroll position
    const tocLinks = document.querySelectorAll('.docs-toc-list a');
    
    if (tocLinks.length > 0) {
        window.addEventListener('scroll', function() {
            const scrollPosition = window.scrollY;
            
            // Find the current section
            let currentSection = null;
            const sections = Array.from(tocLinks).map(link => {
                const sectionId = link.getAttribute('href').substring(1);
                const section = document.getElementById(sectionId);
                return { 
                    id: sectionId, 
                    offset: section.offsetTop,
                    link: link
                };
            });
            
            sections.forEach(section => {
                if (scrollPosition >= section.offset - 100) {
                    currentSection = section.id;
                }
            });
            
            // Highlight the current section in the TOC
            tocLinks.forEach(link => {
                link.classList.remove('active');
                if (link.getAttribute('href') === `#${currentSection}`) {
                    link.classList.add('active');
                }
            });
        });
    }
    
    // Smooth scrolling for anchor links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            
            const targetId = this.getAttribute('href');
            if (targetId === '#') return;
            
            const target = document.querySelector(targetId);
            if (target) {
                window.scrollTo({
                    top: target.offsetTop - 20,
                    behavior: 'smooth'
                });
                
                // Update URL hash without scrolling
                history.pushState(null, null, targetId);
            }
        });
    });
    
    // Code syntax highlighting using Prism.js (if available)
    if (typeof Prism !== 'undefined') {
        Prism.highlightAll();
    }
    
    // Search functionality (basic implementation)
    const searchInput = document.querySelector('.docs-search input');
    
    if (searchInput) {
        searchInput.addEventListener('input', function() {
            const query = this.value.toLowerCase().trim();
            
            if (query.length < 2) {
                // Reset all items to visible
                document.querySelectorAll('.docs-menu li').forEach(item => {
                    item.style.display = '';
                });
                return;
            }
            
            // Filter menu items based on search query
            document.querySelectorAll('.docs-menu li').forEach(item => {
                const linkText = item.querySelector('a').textContent.toLowerCase();
                if (linkText.includes(query)) {
                    item.style.display = '';
                } else {
                    item.style.display = 'none';
                }
            });
        });
    }
});