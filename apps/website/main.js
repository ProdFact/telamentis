document.addEventListener('DOMContentLoaded', function() {
    // Smooth scrolling for anchor links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            
            const target = document.querySelector(this.getAttribute('href'));
            if (target) {
                target.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
            }
        });
    });
    
    // Check if this is a documentation page
    const isDocsPage = window.location.pathname.includes('/docs/');
    
    // Initialize docs-specific functionality if needed
    if (isDocsPage) {
        // Table of contents generation
        const content = document.querySelector('.docs-content');
        const toc = document.querySelector('.docs-toc-list');
        
        if (content && toc) {
            const headings = content.querySelectorAll('h2, h3');
            
            headings.forEach(heading => {
                // Create ID if not exists
                if (!heading.id) {
                    heading.id = heading.textContent
                        .toLowerCase()
                        .replace(/[^\w\s-]/g, '')
                        .replace(/\s+/g, '-');
                }
                
                const item = document.createElement('li');
                const link = document.createElement('a');
                
                link.href = `#${heading.id}`;
                link.textContent = heading.textContent;
                
                if (heading.tagName === 'H3') {
                    item.style.paddingLeft = '1rem';
                }
                
                item.appendChild(link);
                toc.appendChild(item);
            });
        }
    }
    
    
    // Add animation classes on scroll
    const animateElements = document.querySelectorAll('.feature-card, .timeline-item, .community-link');
    
    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('animate');
            }
        });
    }, {
        threshold: 0.1
    });
    
    animateElements.forEach(element => {
        observer.observe(element);
    });
    
    // Mobile menu toggle
    const mobileMenuToggle = document.createElement('button');
    mobileMenuToggle.className = 'mobile-menu-toggle';
    mobileMenuToggle.innerHTML = '<i class="fas fa-bars"></i>';
    
    const nav = document.querySelector('nav');
    const navLinks = document.querySelector('nav ul');
    
    if (window.innerWidth <= 768) {
        nav.appendChild(mobileMenuToggle);
        navLinks.classList.add('hidden');
    }
    
    mobileMenuToggle.addEventListener('click', () => {
        navLinks.classList.toggle('hidden');
        mobileMenuToggle.innerHTML = navLinks.classList.contains('hidden') 
            ? '<i class="fas fa-bars"></i>' 
            : '<i class="fas fa-times"></i>';
    });
    
    window.addEventListener('resize', () => {
        if (window.innerWidth > 768) {
            if (navLinks.classList.contains('hidden')) {
                navLinks.classList.remove('hidden');
            }
            if (nav.contains(mobileMenuToggle)) {
                nav.removeChild(mobileMenuToggle);
            }
        } else {
            if (!nav.contains(mobileMenuToggle)) {
                nav.appendChild(mobileMenuToggle);
            }
        }
    });
});