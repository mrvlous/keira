// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

// Theme Management (Synchronized Navbar Toggle & Drawer Segmented Control)
const themeToggle = document.getElementById('themeToggle');
const themeStatusLabel = document.getElementById('themeStatusLabel');
const segLight = document.getElementById('themeSegLight');
const segDark = document.getElementById('themeSegDark');
const segAuto = document.getElementById('themeSegAuto');

function getSystemTheme(){
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function updateThemeUI(prefTheme){
  const effectiveTheme = prefTheme === 'auto' ? getSystemTheme() : prefTheme;
  if (prefTheme === 'auto'){
    document.documentElement.removeAttribute('data-theme');
  } else {
    document.documentElement.setAttribute('data-theme', prefTheme);
  }

  // Update theme toggle tooltip/label
  if (themeToggle){
    const nextTheme = effectiveTheme === 'dark' ? 'Light' : 'Dark';
    themeToggle.setAttribute('title', `Switch to ${nextTheme} Mode`);
    themeToggle.setAttribute('aria-label', `Switch to ${nextTheme} Mode`);
  }

  // Update drawer status label
  if (themeStatusLabel){
    if (prefTheme === 'auto'){
      themeStatusLabel.textContent = `Auto (${effectiveTheme === 'dark' ? 'Dark' : 'Light'})`;
    } else {
      themeStatusLabel.textContent = effectiveTheme === 'dark' ? 'Dark Mode' : 'Light Mode';
    }
  }

  // Update segmented control buttons
  if (segLight) segLight.classList.toggle('active', prefTheme === 'light');
  if (segDark) segDark.classList.toggle('active', prefTheme === 'dark');
  if (segAuto) segAuto.classList.toggle('active', prefTheme === 'auto');
}

function setTheme(prefTheme){
  localStorage.setItem('keira-theme', prefTheme);
  updateThemeUI(prefTheme);
}

// Initialize theme
const savedTheme = localStorage.getItem('keira-theme') || 'auto';
updateThemeUI(savedTheme);

// Listen to system theme change if auto
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
  const currentPref = localStorage.getItem('keira-theme') || 'auto';
  if (currentPref === 'auto'){
    updateThemeUI('auto');
  }
});

// Navbar toggle click (toggles between light and dark)
if (themeToggle){
  themeToggle.addEventListener('click', () => {
    const cur = document.documentElement.getAttribute('data-theme');
    const isDark = cur ? cur === 'dark' : getSystemTheme() === 'dark';
    const next = isDark ? 'light' : 'dark';
    setTheme(next);
  });
}

// Drawer Segmented Control buttons
if (segLight) segLight.addEventListener('click', () => setTheme('light'));
if (segDark) segDark.addEventListener('click', () => setTheme('dark'));
if (segAuto) segAuto.addEventListener('click', () => setTheme('auto'));

// Mobile navigation drawer toggle with body scroll lock
const menuToggle = document.getElementById('menuToggle');
const mobileDrawer = document.getElementById('mobileDrawer');
const mobileDrawerBackdrop = document.getElementById('mobileDrawerBackdrop');

function closeMobileMenu(){
  if (mobileDrawer) mobileDrawer.classList.remove('open');
  if (mobileDrawerBackdrop) mobileDrawerBackdrop.classList.remove('open');
  if (menuToggle) menuToggle.setAttribute('aria-expanded', 'false');
  document.body.classList.remove('menu-open');
}

function openMobileMenu(){
  if (mobileDrawer) mobileDrawer.classList.add('open');
  if (mobileDrawerBackdrop) mobileDrawerBackdrop.classList.add('open');
  if (menuToggle) menuToggle.setAttribute('aria-expanded', 'true');
  document.body.classList.add('menu-open');
}

if (menuToggle){
  menuToggle.addEventListener('click', () => {
    const isOpen = mobileDrawer && mobileDrawer.classList.contains('open');
    if (isOpen) closeMobileMenu();
    else openMobileMenu();
  });
}

if (mobileDrawerBackdrop){
  mobileDrawerBackdrop.addEventListener('click', closeMobileMenu);
}

if (mobileDrawer){
  mobileDrawer.querySelectorAll('.drawer-link').forEach(link => {
    link.addEventListener('click', closeMobileMenu);
  });
}

// Hero command chip click-to-copy
const heroCmdChip = document.getElementById('heroCmdChip');
const heroCmdBtn = document.getElementById('heroCmdCopyBtn');
function copyHeroCommand(){
  if (!navigator.clipboard) return;
  navigator.clipboard.writeText('make test-all').then(() => {
    if (heroCmdChip) heroCmdChip.classList.add('copied');
    setTimeout(() => {
      if (heroCmdChip) heroCmdChip.classList.remove('copied');
    }, 1600);
  }).catch(() => {});
}
if (heroCmdChip) heroCmdChip.addEventListener('click', copyHeroCommand);
if (heroCmdBtn) heroCmdBtn.addEventListener('click', (e) => {
  e.stopPropagation();
  copyHeroCommand();
});

// Quick start code copy button
const copyQuickStart = document.getElementById('copyQuickStart');
const quickStartCode = document.getElementById('quickStartCode');
if (copyQuickStart && quickStartCode){
  copyQuickStart.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(quickStartCode.innerText.trim());
      copyQuickStart.classList.add('copied');
      const label = copyQuickStart.querySelector('.copy-label');
      if (label) label.textContent = 'Copied!';
      setTimeout(() => {
        copyQuickStart.classList.remove('copied');
        if (label) label.textContent = 'Copy';
      }, 1600);
    } catch {}
  });
}

// Card 3D parallax on mouse move (desktop only)
if (window.matchMedia('(hover: hover)').matches){
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
}

// Gallery strip drag-to-scroll & arrow navigation
const strip = document.getElementById('strip');
if (strip){
  let isDown = false;
  let startX = 0;
  let scrollLeft = 0;

  strip.addEventListener('mousedown', (e) => {
    isDown = true;
    strip.classList.add('dragging');
    startX = e.pageX - strip.offsetLeft;
    scrollLeft = strip.scrollLeft;
  });

  strip.addEventListener('mouseleave', () => {
    isDown = false;
    strip.classList.remove('dragging');
  });

  strip.addEventListener('mouseup', () => {
    isDown = false;
    strip.classList.remove('dragging');
  });

  strip.addEventListener('mousemove', (e) => {
    if (!isDown) return;
    e.preventDefault();
    const x = e.pageX - strip.offsetLeft;
    const walk = (x - startX) * 1.5;
    strip.scrollLeft = scrollLeft - walk;
  });

  strip.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowRight') strip.scrollBy({ left: 300, behavior: 'smooth' });
    if (e.key === 'ArrowLeft') strip.scrollBy({ left: -300, behavior: 'smooth' });
  });
}

// Lightbox with counter, mobile thumb nav, touch swipe & keyboard
const lb = document.getElementById('lightbox');
const lbImg = document.getElementById('lbImg');
const lbCaption = document.getElementById('lbCaption');
const lbCounter = document.getElementById('lbCounter');
const lbClose = document.getElementById('lbClose');
const lbPrev = document.getElementById('lbPrev');
const lbNext = document.getElementById('lbNext');
const shots = Array.from(document.querySelectorAll('.shot'));
let currentIdx = 0;

function showLightboxImage(idx){
  if (idx < 0) idx = shots.length - 1;
  if (idx >= shots.length) idx = 0;
  currentIdx = idx;
  const shot = shots[currentIdx];
  if (!shot || !lbImg) return;
  const img = shot.querySelector('img');
  const caption = shot.getAttribute('data-caption') || shot.querySelector('figcaption')?.innerText || '';
  if (img) lbImg.src = img.src;
  if (lbCaption) lbCaption.innerText = caption;
  if (lbCounter) lbCounter.innerText = `${currentIdx + 1} / ${shots.length}`;
}

shots.forEach((shot, i) => {
  shot.addEventListener('click', () => {
    if (!lb) return;
    showLightboxImage(i);
    lb.classList.add('open');
    document.body.classList.add('menu-open');
  });
});

function closeLightbox(){
  if (lb) lb.classList.remove('open');
  document.body.classList.remove('menu-open');
}

if (lbClose) lbClose.addEventListener('click', closeLightbox);
if (lbPrev) lbPrev.addEventListener('click', (e) => {
  e.stopPropagation();
  showLightboxImage(currentIdx - 1);
});
if (lbNext) lbNext.addEventListener('click', (e) => {
  e.stopPropagation();
  showLightboxImage(currentIdx + 1);
});

if (lb){
  lb.addEventListener('click', (e) => {
    if (e.target === lb) closeLightbox();
  });
}

// Keyboard shortcuts for Lightbox & Menu
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape'){
    closeLightbox();
    closeMobileMenu();
  }
  if (lb && lb.classList.contains('open')){
    if (e.key === 'ArrowRight') showLightboxImage(currentIdx + 1);
    if (e.key === 'ArrowLeft') showLightboxImage(currentIdx - 1);
  }
});

// Mobile touch swipe gestures inside Lightbox
if (lb){
  let touchStartX = 0;
  let touchEndX = 0;

  lb.addEventListener('touchstart', (e) => {
    touchStartX = e.changedTouches[0].screenX;
  }, { passive: true });

  lb.addEventListener('touchend', (e) => {
    touchEndX = e.changedTouches[0].screenX;
    const diff = touchEndX - touchStartX;
    if (Math.abs(diff) > 45){
      if (diff < 0) showLightboxImage(currentIdx + 1);
      else showLightboxImage(currentIdx - 1);
    }
  }, { passive: true });
}
