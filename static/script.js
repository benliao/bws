// BWS Static Website JavaScript

document.addEventListener('DOMContentLoaded', function() {
    console.log('BWS Static Website loaded!');
    
    // Add some interactive features
    addDateToFooter();
    highlightCurrentPage();
});

function addDateToFooter() {
    const footer = document.querySelector('footer p');
    if (footer) {
        const currentYear = new Date().getFullYear();
        footer.innerHTML = footer.innerHTML.replace('2025', currentYear);
    }
}

function highlightCurrentPage() {
    const currentPath = window.location.pathname;
    const navLinks = document.querySelectorAll('nav a');
    
    navLinks.forEach(link => {
        const linkPath = new URL(link.href).pathname;
        if (linkPath === currentPath || 
            (currentPath === '/' && linkPath.endsWith('index.html'))) {
            link.style.backgroundColor = '#34495e';
            link.style.fontWeight = 'bold';
        }
    });
}

function handleSubmit(event) {
    event.preventDefault();
    
    const formData = new FormData(event.target);
    const data = {
        name: formData.get('name'),
        email: formData.get('email'),
        message: formData.get('message')
    };
    
    // Simulate form submission to the API
    submitContactForm(data);
}

async function submitContactForm(data) {
    const resultDiv = document.getElementById('form-result');
    
    try {
        // In a real application, you would send this to your API
        // For now, we'll simulate a successful submission
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        resultDiv.innerHTML = `
            <div class="success">
                <strong>Thank you, ${data.name}!</strong><br>
                Your message has been received. We'll get back to you at ${data.email}.
            </div>
        `;
        resultDiv.style.display = 'block';
        
        // Clear the form
        event.target.reset();
        
    } catch (error) {
        resultDiv.innerHTML = `
            <div class="error">
                <strong>Error:</strong> Failed to send message. Please try again later.
            </div>
        `;
        resultDiv.style.display = 'block';
    }
}

// API interaction example
async function checkServerHealth() {
    try {
        const response = await fetch('/api/health');
        const data = await response.json();
        console.log('Server health:', data);
        return data;
    } catch (error) {
        console.error('Failed to check server health:', error);
        return null;
    }
}

// Add a health check indicator to pages with API links
document.addEventListener('DOMContentLoaded', async function() {
    const healthLink = document.querySelector('a[href="/api/health"]');
    if (healthLink) {
        const health = await checkServerHealth();
        if (health && health.status === 'ok') {
            healthLink.innerHTML += ' ✅';
            healthLink.title = 'API is healthy';
        } else {
            healthLink.innerHTML += ' ❌';
            healthLink.title = 'API is not responding';
        }
    }
});
