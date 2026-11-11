// SPDX-License-Identifier: GPL-2.0-only

/**
 * Keira Kernel Documentation Interactive Client Script
 * Path: docs/sphinx-static/custom.js
 *
 * Implements code block copy buttons, search keyboard shortcuts
 * and responsive table auto-wrapping.
 */

document.addEventListener("DOMContentLoaded", function () {
    // 1. Add Copy to Clipboard buttons on all code blocks
    const codeBlocks = document.querySelectorAll("div[class^='highlight']");
    codeBlocks.forEach(function (block) {
        const pre = block.querySelector("pre");
        if (!pre) return;

        const btn = document.createElement("button");
        btn.className = "keira-copy-btn";
        btn.type = "button";
        btn.innerText = "Copy";
        btn.setAttribute("title", "Copy snippet to clipboard");

        btn.addEventListener("click", function () {
            const text = pre.innerText;
            navigator.clipboard.writeText(text).then(
                function () {
                    btn.innerText = "Copied!";
                    btn.classList.add("copied");
                    setTimeout(function () {
                        btn.innerText = "Copy";
                        btn.classList.remove("copied");
                    }, 2000);
                },
                function (err) {
                    console.error("Clipboard copy failed: ", err);
                    btn.innerText = "Failed";
                    setTimeout(function () {
                        btn.innerText = "Copy";
                    }, 2000);
                }
            );
        });

        block.appendChild(btn);
    });

    // 2. Keyboard shortcut: '/' focuses the Sphinx search input
    document.addEventListener("keydown", function (e) {
        const activeTag = document.activeElement ? document.activeElement.tagName.toLowerCase() : "";
        if (activeTag === "input" || activeTag === "textarea" || activeTag === "select") {
            return;
        }

        if (e.key === "/" && !e.ctrlKey && !e.metaKey && !e.altKey) {
            const searchInput = document.querySelector(".wy-side-nav-search input[type='text']");
            if (searchInput) {
                e.preventDefault();
                searchInput.focus();
                searchInput.select();
            }
        }
    });

    // 3. Auto-wrap tables into responsive containers
    const tables = document.querySelectorAll(".rst-content table.docutils");
    tables.forEach(function (table) {
        if (!table.parentElement.classList.contains("keira-table-wrap")) {
            const wrapper = document.createElement("div");
            wrapper.className = "keira-table-wrap";
            table.parentNode.insertBefore(wrapper, table);
            wrapper.appendChild(table);
        }
    });
});

