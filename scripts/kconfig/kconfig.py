#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-only
#
# Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
#
# Freestanding Linux-Grade Kconfig Parser and Configuration Model for Keira Kernel

import os
import re
import sys

class Symbol:
    def __init__(self, name):
        self.name = name
        self.type = "bool"  # bool, int, string, hex, tristate
        self.prompt = None
        self.defaults = []  # list of (value, condition)
        self.depends_on = []
        self.selects = []
        self.help_text = ""
        self.range = None  # (min, max)
        self.value = None
        self.choice = None
        self.menu = None

    def get_effective_default(self, kconfig):
        for val, cond in self.defaults:
            if not cond or kconfig.eval_condition(cond):
                return val
        if self.type == "bool":
            return False
        elif self.type == "int":
            return 0
        elif self.type == "string":
            return ""
        return None

    def is_visible(self, kconfig):
        for dep in self.depends_on:
            if not kconfig.eval_condition(dep):
                return False
        if self.menu and not self.menu.is_visible(kconfig):
            return False
        return True

    def get_value(self, kconfig):
        if not self.is_visible(kconfig):
            return False if self.type == "bool" else ""
        if self.value is not None:
            return self.value
        return self.get_effective_default(kconfig)

class Choice:
    def __init__(self, prompt=None):
        self.prompt = prompt
        self.items = []
        self.default_name = None
        self.depends_on = []
        self.menu = None

    def is_visible(self, kconfig):
        for dep in self.depends_on:
            if not kconfig.eval_condition(dep):
                return False
        if self.menu and not self.menu.is_visible(kconfig):
            return False
        return True

    def get_selected(self, kconfig):
        # Return currently selected symbol in choice
        for sym in self.items:
            if sym.value is True:
                return sym
        # Default fallback
        if self.default_name:
            sym = kconfig.symbols.get(self.default_name)
            if sym and sym in self.items:
                return sym
        if self.items:
            return self.items[0]
        return None

class Menu:
    def __init__(self, title, parent=None):
        self.title = title
        self.parent = parent
        self.items = []  # Symbols, Choices, Submenus
        self.depends_on = []

    def is_visible(self, kconfig):
        for dep in self.depends_on:
            if not kconfig.eval_condition(dep):
                return False
        if self.parent and not self.parent.is_visible(kconfig):
            return False
        return True

class Kconfig:
    def __init__(self, root_file="Kconfig"):
        self.root_file = root_file
        self.mainmenu = "Keira Kernel Configuration"
        self.symbols = {}
        self.root_menu = Menu("Root")
        self.parsed_files = set()
        if os.path.exists(root_file):
            self.parse_file(root_file, self.root_menu)

    def eval_condition(self, expr):
        if not expr:
            return True
        tokens = expr.split()
        if len(tokens) == 1:
            token = tokens[0]
            if token.startswith("!"):
                sub = token[1:]
                sym = self.symbols.get(sub)
                if not sym:
                    return True
                val = sym.value if sym.value is not None else sym.get_effective_default(self)
                return not bool(val)
            sym = self.symbols.get(token)
            if not sym:
                return False
            val = sym.value if sym.value is not None else sym.get_effective_default(self)
            return bool(val)
        # Simple logical AND / OR handling
        if "&&" in tokens:
            idx = tokens.index("&&")
            left = " ".join(tokens[:idx])
            right = " ".join(tokens[idx+1:])
            return self.eval_condition(left) and self.eval_condition(right)
        if "||" in tokens:
            idx = tokens.index("||")
            left = " ".join(tokens[:idx])
            right = " ".join(tokens[idx+1:])
            return self.eval_condition(left) or self.eval_condition(right)
        return True

    def parse_file(self, filepath, current_menu):
        filepath = os.path.normpath(filepath)
        if filepath in self.parsed_files:
            return
        self.parsed_files.add(filepath)

        if not os.path.exists(filepath):
            return

        with open(filepath, "r", encoding="utf-8") as f:
            lines = f.readlines()

        current_sym = None
        current_choice = None
        menu_stack = [current_menu]
        in_help = False
        help_indent = 0

        i = 0
        while i < len(lines):
            line = lines[i]
            raw_line = line.rstrip("\n")
            stripped = raw_line.strip()

            if in_help:
                if stripped and (len(raw_line) - len(raw_line.lstrip())) >= help_indent:
                    if current_sym:
                        current_sym.help_text += stripped + "\n"
                    i += 1
                    continue
                else:
                    in_help = False

            if not stripped or stripped.startswith("#"):
                i += 1
                continue

            # Directives
            if stripped.startswith("mainmenu "):
                self.mainmenu = stripped.split("mainmenu", 1)[1].strip().strip('"')
            elif stripped.startswith("source "):
                src_path = stripped.split("source", 1)[1].strip().strip('"')
                if not os.path.isabs(src_path):
                    src_path = os.path.join(os.path.dirname(filepath), src_path)
                self.parse_file(src_path, menu_stack[-1])
            elif stripped.startswith("menu "):
                title = stripped.split("menu", 1)[1].strip().strip('"')
                new_menu = Menu(title, parent=menu_stack[-1])
                menu_stack[-1].items.append(new_menu)
                menu_stack.append(new_menu)
                current_sym = None
            elif stripped == "endmenu":
                if len(menu_stack) > 1:
                    menu_stack.pop()
                current_sym = None
            elif stripped.startswith("choice"):
                new_choice = Choice()
                current_choice = new_choice
                menu_stack[-1].items.append(new_choice)
                current_sym = None
            elif stripped == "endchoice":
                current_choice = None
                current_sym = None
            elif stripped.startswith("config "):
                sym_name = stripped.split()[1]
                if sym_name not in self.symbols:
                    self.symbols[sym_name] = Symbol(sym_name)
                current_sym = self.symbols[sym_name]
                current_sym.menu = menu_stack[-1]
                if current_choice:
                    current_choice.items.append(current_sym)
                    current_sym.choice = current_choice
                else:
                    menu_stack[-1].items.append(current_sym)
            elif stripped.startswith("prompt ") and current_choice:
                current_choice.prompt = stripped.split("prompt", 1)[1].strip().strip('"')
            elif stripped.startswith("default ") and current_choice:
                current_choice.default_name = stripped.split("default", 1)[1].strip().split()[0]
            elif current_sym:
                if stripped.startswith("bool"):
                    current_sym.type = "bool"
                    parts = stripped.split("bool", 1)[1].strip()
                    if parts:
                        current_sym.prompt = parts.strip('"')
                elif stripped.startswith("int"):
                    current_sym.type = "int"
                    parts = stripped.split("int", 1)[1].strip()
                    if parts:
                        current_sym.prompt = parts.strip('"')
                elif stripped.startswith("string"):
                    current_sym.type = "string"
                    parts = stripped.split("string", 1)[1].strip()
                    if parts:
                        current_sym.prompt = parts.strip('"')
                elif stripped.startswith("hex"):
                    current_sym.type = "hex"
                    parts = stripped.split("hex", 1)[1].strip()
                    if parts:
                        current_sym.prompt = parts.strip('"')
                elif stripped.startswith("prompt "):
                    current_sym.prompt = stripped.split("prompt", 1)[1].strip().strip('"')
                elif stripped.startswith("default "):
                    def_expr = stripped.split("default", 1)[1].strip()
                    val = def_expr
                    cond = None
                    if " if " in def_expr:
                        val, cond = def_expr.split(" if ", 1)
                    val = val.strip().strip('"')
                    if current_sym.type == "bool":
                        val = (val == "y" or val == "1" or val == "true")
                    elif current_sym.type == "int":
                        try:
                            val = int(val)
                        except:
                            val = 0
                    current_sym.defaults.append((val, cond))
                elif stripped.startswith("depends on "):
                    dep = stripped.split("depends on", 1)[1].strip()
                    current_sym.depends_on.append(dep)
                elif stripped.startswith("select "):
                    sel = stripped.split("select", 1)[1].strip()
                    current_sym.selects.append(sel)
                elif stripped.startswith("range "):
                    rparts = stripped.split()[1:]
                    if len(rparts) >= 2:
                        try:
                            current_sym.range = (int(rparts[0]), int(rparts[1]))
                        except:
                            pass
                elif stripped == "help":
                    in_help = True
                    help_indent = len(raw_line) - len(raw_line.lstrip()) + 2

            i += 1

    def load_config(self, config_path):
        if not os.path.exists(config_path):
            return False
        with open(config_path, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    # Check for "# CONFIG_XYZ is not set"
                    m = re.match(r"^#\s*CONFIG_([A-Za-z0-9_]+)\s+is not set", line)
                    if m:
                        sym_name = m.group(1)
                        if sym_name in self.symbols:
                            self.symbols[sym_name].value = False
                    continue
                if "=" in line:
                    key, val = line.split("=", 1)
                    key = key.strip()
                    val = val.strip()
                    if key.startswith("CONFIG_"):
                        sym_name = key[len("CONFIG_"):]
                        if sym_name not in self.symbols:
                            self.symbols[sym_name] = Symbol(sym_name)
                        sym = self.symbols[sym_name]
                        if val == "y":
                            sym.value = True
                        elif val == "n":
                            sym.value = False
                        elif val.startswith('"') and val.endswith('"'):
                            sym.value = val[1:-1]
                        else:
                            try:
                                sym.value = int(val)
                            except:
                                sym.value = val
        return True

    def write_config(self, config_path):
        with open(config_path, "w", encoding="utf-8") as f:
            f.write("#\n# Automatically generated file; DO NOT EDIT.\n")
            f.write(f"# {self.mainmenu}\n#\n\n")

            def dump_menu(menu):
                if menu != self.root_menu:
                    f.write(f"\n#\n# {menu.title}\n#\n")
                for item in menu.items:
                    if isinstance(item, Menu):
                        dump_menu(item)
                    elif isinstance(item, Choice):
                        f.write(f"#\n# {item.prompt or 'Choice'}\n#\n")
                        selected = item.get_selected(self)
                        for sym in item.items:
                            if sym == selected:
                                f.write(f"CONFIG_{sym.name}=y\n")
                            else:
                                f.write(f"# CONFIG_{sym.name} is not set\n")
                    elif isinstance(item, Symbol):
                        val = item.get_value(self)
                        if item.type == "bool":
                            if val:
                                f.write(f"CONFIG_{item.name}=y\n")
                            else:
                                f.write(f"# CONFIG_{item.name} is not set\n")
                        elif item.type == "int":
                            f.write(f"CONFIG_{item.name}={val}\n")
                        elif item.type == "string":
                            f.write(f'CONFIG_{item.name}="{val}"\n')
                        elif item.type == "hex":
                            f.write(f"CONFIG_{item.name}={hex(val) if isinstance(val, int) else val}\n")

            dump_menu(self.root_menu)

    def write_autoconf(self, header_path):
        os.makedirs(os.path.dirname(header_path), exist_ok=True)
        with open(header_path, "w", encoding="utf-8") as f:
            f.write("/*\n * Automatically generated file; DO NOT EDIT.\n")
            f.write(f" * {self.mainmenu}\n */\n\n")
            f.write("#ifndef _KEIRA_AUTOCONF_H\n#define _KEIRA_AUTOCONF_H\n\n")

            for sym_name, sym in sorted(self.symbols.items()):
                val = sym.get_value(self)
                if sym.type == "bool" and val:
                    f.write(f"#define CONFIG_{sym_name} 1\n")
                elif sym.type == "int" and val is not None:
                    f.write(f"#define CONFIG_{sym_name} {val}\n")
                elif sym.type == "string" and val is not None:
                    f.write(f'#define CONFIG_{sym_name} "{val}"\n')

            f.write("\n#endif /* _KEIRA_AUTOCONF_H */\n")
