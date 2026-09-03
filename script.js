// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

// Theme toggle with localStorage
const toggle = document.getElementById('themeToggle');
const saved = localStorage.getItem('keira-theme');
if (saved) document.documentElement.setAttribute('data-theme', saved);
if (toggle){
  toggle.addEventListener('click', () => {
    const cur = document.documentElement.getAttribute('data-theme');
    const isDark = cur ? cur === 'dark' : window.matchMedia('(prefers-color-scheme: dark)').matches;
    const next = isDark ? 'light' : 'dark';
    document.documentElement.setAttribute('data-theme', next);
    localStorage.setItem('keira-theme', next);
  });
}

// Card parallax
const cards = document.querySelectorAll('.card');
cards.forEach(card => {
  card.addEventListener('mousemove', (e) => {
    const rect = card.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    card.style.setProperty('--mx', x + 'px');
    card.style.setProperty('--my', y + 'px');
  });
});

// Copy code
document.querySelectorAll('pre code').forEach(el => {
  el.style.cursor = 'pointer';
  el.title = 'Click to copy';
  el.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(el.innerText);
      const prev = el.innerText;
      el.innerText = 'Copied to clipboard';
      setTimeout(() => el.innerText = prev, 1200);
    } catch {}
  });
});

// Lightbox with keyboard
const lb = document.getElementById('lightbox');
const lbImg = lb ? lb.querySelector('img') : null;
document.querySelectorAll('.shot img').forEach(img => {
  img.addEventListener('click', () => {
    if (!lb || !lbImg) return;
    lbImg.src = img.src;
    lb.classList.add('open');
  });
});
if (lb){
  lb.addEventListener('click', () => lb.classList.remove('open'));
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') lb.classList.remove('open');
    if (e.key === 'ArrowRight' && lb.classList.contains('open')){
      const cur = [...document.querySelectorAll('.shot img')].findIndex(i => i.src === lbImg.src);
      const next = document.querySelectorAll('.shot img')[cur + 1];
      if (next) lbImg.src = next.src;
    }
    if (e.key === 'ArrowLeft' && lb.classList.contains('open')){
      const cur = [...document.querySelectorAll('.shot img')].findIndex(i => i.src === lbImg.src);
      const prev = document.querySelectorAll('.shot img')[cur - 1];
      if (prev) lbImg.src = prev.src;
    }
  });
}
// Gallery keyboard scroll
const strip = document.getElementById('strip');
if (strip){
  strip.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowRight') strip.scrollBy({ left: 280, behavior: 'smooth' });
    if (e.key === 'ArrowLeft') strip.scrollBy({ left: -280, behavior: 'smooth' });
  });
}
