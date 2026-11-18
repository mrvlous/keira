#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-only
#
# Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
#
# Interactive Terminal Menuconfig TUI (curses based) for Keira Kernel

import curses
import os
import sys

from kconfig import Choice, Kconfig, Menu, Symbol

class MenuconfigTUI:
    def __init__(self, kconf, config_file=".config"):
        self.kconf = kconf
        self.config_file = config_file
        self.menu_stack = [self.kconf.root_menu]
        self.cursor_stack = [0]
        self.changed = False

    def run(self, stdscr):
        curses.curs_set(0)
        curses.use_default_colors()
        curses.init_pair(1, curses.COLOR_WHITE, curses.COLOR_BLUE)    # Header & border
        curses.init_pair(2, curses.COLOR_BLACK, curses.COLOR_CYAN)    # Highlighted item
        curses.init_pair(3, curses.COLOR_WHITE, -1)                   # Normal item
        curses.init_pair(4, curses.COLOR_YELLOW, -1)                  # Arrow & prompt
        curses.init_pair(5, curses.COLOR_GREEN, -1)                   # Active choice

        while True:
            current_menu = self.menu_stack[-1]
            items = [item for item in current_menu.items if item.is_visible(self.kconf)]
            cursor = self.cursor_stack[-1]
            if cursor >= len(items):
                cursor = max(0, len(items) - 1)
                self.cursor_stack[-1] = cursor

            stdscr.clear()
            max_y, max_x = stdscr.getmaxyx()

            # 1. Header Banner
            title = f" Keira Kernel Configuration - {current_menu.title} "
            stdscr.attron(curses.color_pair(1) | curses.A_BOLD)
            stdscr.addstr(0, max(0, (max_x - len(title)) // 2), title[:max_x-1])
            stdscr.attroff(curses.color_pair(1) | curses.A_BOLD)

            # 2. Border Box
            box_top = 2
            box_bottom = max_y - 4
            box_height = max(1, box_bottom - box_top)

            for y in range(box_top, box_bottom + 1):
                stdscr.addch(y, 2, curses.ACS_VLINE)
                stdscr.addch(y, max_x - 3, curses.ACS_VLINE)
            for x in range(2, max_x - 2):
                stdscr.addch(box_top, x, curses.ACS_HLINE)
                stdscr.addch(box_bottom, x, curses.ACS_HLINE)
            stdscr.addch(box_top, 2, curses.ACS_ULCORNER)
            stdscr.addch(box_top, max_x - 3, curses.ACS_URCORNER)
            stdscr.addch(box_bottom, 2, curses.ACS_LLCORNER)
            stdscr.addch(box_bottom, max_x - 3, curses.ACS_LRCORNER)

            # 3. Render Menu Items
            scroll_offset = 0
            visible_rows = box_height - 1
            if cursor >= visible_rows:
                scroll_offset = cursor - visible_rows + 1

            for idx in range(visible_rows):
                item_idx = idx + scroll_offset
                if item_idx >= len(items):
                    break
                item = items[item_idx]
                row_y = box_top + 1 + idx

                is_selected = (item_idx == cursor)
                color = curses.color_pair(2) if is_selected else curses.color_pair(3)

                display_str = ""
                if isinstance(item, Menu):
                    display_str = f"   ---> {item.title}"
                elif isinstance(item, Choice):
                    sel = item.get_selected(self.kconf)
                    sel_name = sel.prompt if sel else "None"
                    display_str = f"  ({sel_name}) {item.prompt or 'Choice'} --->"
                elif isinstance(item, Symbol):
                    val = item.get_value(self.kconf)
                    if item.type == "bool":
                        marker = "[*]" if val else "[ ]"
                        prompt = item.prompt or item.name
                        display_str = f"  {marker} {prompt}"
                    elif item.type == "int":
                        display_str = f"  ({val}) {item.prompt or item.name}"
                    elif item.type == "string":
                        display_str = f'  ("{val}") {item.prompt or item.name}'

                stdscr.attron(color)
                stdscr.addstr(row_y, 4, display_str.ljust(max_x - 8)[:max_x - 8])
                stdscr.attroff(color)

            # 4. Footer Keybindings
            footer = " <Enter> Select/Enter  <Space> Toggle  <h/?> Help  <q/Esc> Exit "
            stdscr.attron(curses.color_pair(1))
            stdscr.addstr(max_y - 2, max(0, (max_x - len(footer)) // 2), footer[:max_x-1])
            stdscr.attroff(curses.color_pair(1))

            stdscr.refresh()

            # Handle Input
            key = stdscr.getch()

            if key in (curses.KEY_UP, ord('k')):
                if cursor > 0:
                    self.cursor_stack[-1] -= 1
            elif key in (curses.KEY_DOWN, ord('j')):
                if cursor < len(items) - 1:
                    self.cursor_stack[-1] += 1
            elif key in (curses.KEY_ENTER, 10, 13, ord(' ')):
                if items:
                    selected_item = items[cursor]
                    if isinstance(selected_item, Menu):
                        self.menu_stack.append(selected_item)
                        self.cursor_stack.append(0)
                    elif isinstance(selected_item, Choice):
                        # Cycle choice selection
                        curr_sel = selected_item.get_selected(self.kconf)
                        idx_sel = selected_item.items.index(curr_sel) if curr_sel in selected_item.items else -1
                        next_sel = selected_item.items[(idx_sel + 1) % len(selected_item.items)]
                        for sym in selected_item.items:
                            sym.value = (sym == next_sel)
                        self.changed = True
                    elif isinstance(selected_item, Symbol):
                        if selected_item.type == "bool":
                            val = selected_item.get_value(self.kconf)
                            selected_item.value = not val
                            self.changed = True
                        elif selected_item.type == "int":
                            # Cycle within range or toggle
                            val = selected_item.get_value(self.kconf)
                            new_val = val + 1
                            if selected_item.range and new_val > selected_item.range[1]:
                                new_val = selected_item.range[0]
                            selected_item.value = new_val
                            self.changed = True
            elif key in (ord('h'), ord('?')):
                if items:
                    selected_item = items[cursor]
                    self.show_help(stdscr, selected_item)
            elif key in (27, ord('q')):  # Escape or q
                if len(self.menu_stack) > 1:
                    self.menu_stack.pop()
                    self.cursor_stack.pop()
                else:
                    if self.prompt_exit(stdscr):
                        break

    def show_help(self, stdscr, item):
        max_y, max_x = stdscr.getmaxyx()
        help_text = getattr(item, "help_text", "")
        if not help_text:
            help_text = "No detailed help available for this configuration symbol."

        win = curses.newwin(max_y - 8, max_x - 10, 4, 5)
        win.box()
        win.addstr(1, 2, f" Help: {getattr(item, 'prompt', getattr(item, 'name', 'Symbol'))} ", curses.A_BOLD)

        lines = help_text.strip().split("\n")
        for idx, line in enumerate(lines[:max_y - 12]):
            win.addstr(3 + idx, 3, line[:max_x - 16])

        win.addstr(max_y - 10, max_x // 2 - 10, "[ Press any key ]", curses.A_REVERSE)
        win.refresh()
        win.getch()

    def prompt_exit(self, stdscr):
        max_y, max_x = stdscr.getmaxyx()
        win = curses.newwin(6, 50, (max_y - 6) // 2, (max_x - 50) // 2)
        win.box()
        msg = "Save configuration changes?"
        win.addstr(2, (50 - len(msg)) // 2, msg, curses.A_BOLD)
        win.addstr(4, 12, "[ Yes (<y>) ]", curses.A_REVERSE)
        win.addstr(4, 28, "[ No (<n>) ]")
        win.refresh()

        while True:
            k = win.getch()
            if k in (ord('y'), ord('Y'), 10, 13):
                self.kconf.write_config(self.config_file)
                self.kconf.write_autoconf("include/generated/autoconf.h")
                return True
            elif k in (ord('n'), ord('N'), 27, ord('q')):
                return True

def main():
    kconfig_file = sys.argv[1] if len(sys.argv) > 1 else "Kconfig"
    kconf = Kconfig(kconfig_file)
    kconf.load_config(".config")

    if not sys.stdin.isatty():
        print("[KCONFIG] Non-interactive environment detected; running batch validation...")
        kconf.write_config(".config")
        kconf.write_autoconf("include/generated/autoconf.h")
        return

    try:
        tui = MenuconfigTUI(kconf, ".config")
        curses.wrapper(tui.run)
    except Exception as e:
        print(f"[ERR] Menuconfig TUI encountered error: {e}")
        kconf.write_config(".config")
        kconf.write_autoconf("include/generated/autoconf.h")

if __name__ == "__main__":
    main()
