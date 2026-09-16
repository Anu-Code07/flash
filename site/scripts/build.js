#!/usr/bin/env node
/**
 * Vercel build step for the static Flash docs site.
 * No bundler — just validates the site folder is ready to deploy.
 */
const fs = require('fs');
const path = require('path');

const root = path.join(__dirname, '..');
const required = ['index.html', 'css/style.css', 'js/main.js'];

for (const file of required) {
  const full = path.join(root, file);
  if (!fs.existsSync(full)) {
    console.error(`Missing required file: ${file}`);
    process.exit(1);
  }
}

const pages = fs.readdirSync(root).filter((f) => f.endsWith('.html'));
console.log(`Flash docs ready — ${pages.length} HTML pages`);
console.log('Output directory: site/ (static, no compile step)');
