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
const savedTheme = localStorage.getItem('keira-theme');
if (savedTheme) document.documentElement.setAttribute('data-theme', savedTheme);
if (toggle){
  toggle.addEventListener('click', () => {
    const cur = document.documentElement.getAttribute('data-theme');
    const isDark = cur ? cur === 'dark' : window.matchMedia('(prefers-color-scheme: dark)').matches;
    const next = isDark ? 'light' : 'dark';
    document.documentElement.setAttribute('data-theme', next);
    localStorage.setItem('keira-theme', next);
  });
}

// Mobile navigation drawer toggle
const menuToggle = document.getElementById('menuToggle');
const navLinks = document.getElementById('navLinks');
const navBackdrop = document.getElementById('navBackdrop');

function closeMobileMenu(){
  if (navLinks) navLinks.classList.remove('open');
  if (navBackdrop) navBackdrop.classList.remove('open');
  if (menuToggle) menuToggle.setAttribute('aria-expanded', 'false');
}

function openMobileMenu(){
  if (navLinks) navLinks.classList.add('open');
  if (navBackdrop) navBackdrop.classList.add('open');
  if (menuToggle) menuToggle.setAttribute('aria-expanded', 'true');
}

if (menuToggle){
  menuToggle.addEventListener('click', () => {
    const isOpen = navLinks && navLinks.classList.contains('open');
    if (isOpen) closeMobileMenu();
    else openMobileMenu();
  });
}

if (navBackdrop){
  navBackdrop.addEventListener('click', closeMobileMenu);
}

if (navLinks){
  navLinks.querySelectorAll('a').forEach(link => {
    link.addEventListener('click', closeMobileMenu);
  });
}

// Card 3D parallax on pointer movement
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

// Interactive copy button
const copyBtn = document.getElementById('copyQuickStart');
const quickCode = document.getElementById('quickStartCode');
if (copyBtn && quickCode){
  copyBtn.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(quickCode.innerText.trim());
      copyBtn.classList.add('copied');
      const label = copyBtn.querySelector('.copy-label');
      if (label) label.textContent = 'Copied!';
      setTimeout(() => {
        copyBtn.classList.remove('copied');
        if (label) label.textContent = 'Copy';
      }, 1600);
    } catch {}
  });
}

// Desktop mouse drag-to-scroll & keyboard navigation on gallery strip
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

// Lightbox with controls, captions, touch swipe, and keyboard
const lb = document.getElementById('lightbox');
const lbImg = document.getElementById('lbImg');
const lbCaption = document.getElementById('lbCaption');
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
}

shots.forEach((shot, i) => {
  shot.addEventListener('click', () => {
    if (!lb) return;
    showLightboxImage(i);
    lb.classList.add('open');
  });
});

function closeLightbox(){
  if (lb) lb.classList.remove('open');
}

if (lbClose) lbClose.addEventListener('click', closeLightbox);
if (lbPrev) lbPrev.addEventListener('click', () => showLightboxImage(currentIdx - 1));
if (lbNext) lbNext.addEventListener('click', () => showLightboxImage(currentIdx + 1));

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
