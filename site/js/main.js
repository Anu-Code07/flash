// Flash language site — search and navigation helpers

document.addEventListener('DOMContentLoaded', () => {
  // Highlight active nav link
  const path = window.location.pathname.split('/').pop() || 'index.html';
  document.querySelectorAll('.nav-links a, .sidebar a').forEach(a => {
    if (a.getAttribute('href') === path || a.getAttribute('href')?.endsWith(path)) {
      a.classList.add('active');
    }
  });

  // Simple search across fn-items
  const search = document.getElementById('search');
  if (search) {
    search.addEventListener('input', (e) => {
      const q = e.target.value.toLowerCase();
      document.querySelectorAll('.fn-item, .card').forEach(el => {
        const text = el.textContent.toLowerCase();
        el.style.display = text.includes(q) ? '' : 'none';
      });
    });
  }

  // Copy code blocks
  document.querySelectorAll('pre').forEach(pre => {
    pre.style.position = 'relative';
    const btn = document.createElement('button');
    btn.textContent = 'Copy';
    btn.style.cssText = 'position:absolute;top:8px;right:8px;background:#2a2a3a;border:none;color:#8888a0;padding:4px 10px;border-radius:6px;cursor:pointer;font-size:12px;';
    btn.addEventListener('click', () => {
      navigator.clipboard.writeText(pre.textContent);
      btn.textContent = 'Copied!';
      setTimeout(() => btn.textContent = 'Copy', 1500);
    });
    pre.appendChild(btn);
  });
});
