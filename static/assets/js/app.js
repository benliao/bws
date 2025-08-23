// Test JavaScript file in subdirectory
console.log('BWS Test - JavaScript loaded from assets/js/app.js');

document.addEventListener('DOMContentLoaded', function() {
    console.log('DOM content loaded');
    
    // Add some interactive functionality
    const headers = document.querySelectorAll('h1, h2, h3');
    headers.forEach(header => {
        header.addEventListener('click', function() {
            this.style.color = this.style.color === 'blue' ? '' : 'blue';
        });
    });
    
    // Show current time
    const timeElement = document.getElementById('current-time');
    if (timeElement) {
        timeElement.textContent = new Date().toLocaleString();
    }
});
