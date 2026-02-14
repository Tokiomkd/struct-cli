// Initialize Lucide icons when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    lucide.createIcons();
    
    // Initialize scroll animations
    initScrollAnimations();
    
    // Initialize smooth scroll for anchor links
    initSmoothScroll();
});

// Copy to clipboard functionality
function copyCode(text) {
    navigator.clipboard.writeText(text).then(() => {
        // Find the button that triggered this
        const btn = event.target.closest('.copy-btn');
        if (!btn) return;
        
        const originalHTML = btn.innerHTML;
        
        // Show success state
        btn.innerHTML = '<i data-lucide="check" class="w-3 h-3"></i><span>Copied!</span>';
        btn.classList.add('text-github-accent');
        lucide.createIcons();
        
        // Revert after 2 seconds
        setTimeout(() => {
            btn.innerHTML = originalHTML;
            btn.classList.remove('text-github-accent');
            lucide.createIcons();
        }, 2000);
    }).catch(err => {
        console.error('Failed to copy:', err);
    });
}

// Smooth scroll for anchor links
function initSmoothScroll() {
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function(e) {
            e.preventDefault();
            const targetId = this.getAttribute('href');
            const target = document.querySelector(targetId);
            
            if (target) {
                // Account for fixed header height
                const headerOffset = 80;
                const elementPosition = target.getBoundingClientRect().top;
                const offsetPosition = elementPosition + window.pageYOffset - headerOffset;
                
                window.scrollTo({
                    top: offsetPosition,
                    behavior: 'smooth'
                });
            }
        });
    });
}

// Intersection Observer for scroll animations
function initScrollAnimations() {
    const observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };

    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('is-visible');
                // Stop observing once visible
                observer.unobserve(entry.target);
            }
        });
    }, observerOptions);

    // Observe all feature cards for animation
    document.querySelectorAll('.feature-card').forEach((el, index) => {
        // Add stagger delay
        el.style.transitionDelay = `${index * 0.1}s`;
        observer.observe(el);
    });
}

// Handle external link clicks (security)
document.querySelectorAll('a[target="_blank"]').forEach(link => {
    link.addEventListener('click', function(e) {
        // Ensure rel="noopener" is present for security
        if (!this.rel || !this.rel.includes('noopener')) {
            this.rel = this.rel ? `${this.rel} noopener` : 'noopener';
        }
    });
});

// Optional: Keyboard shortcut for search (Cmd/Ctrl + K)
document.addEventListener('keydown', function(e) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        // Focus on the first search-related element or scroll to features
        const featuresSection = document.getElementById('features');
        if (featuresSection) {
            featuresSection.scrollIntoView({ behavior: 'smooth' });
        }
    }
});