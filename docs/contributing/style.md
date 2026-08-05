# Coding Style Standards

This document describes the comment syntax, formatting rules, and linter guidelines required for contributions to Keira Kernel.

## 1. C Source and Header Files
C source files must comply with standard kernel programming rules:
*   **Block Comments Only**: Single-line C++ style comments (`//`) are strictly forbidden. Use C-style block comments (`/* ... */`).
*   **No Inline Comments**: Comments must occupy their own line above the code block they describe. Do not write comments on the same line as code statements.
*   **Documentation Blocks**: All functions, structures, and interfaces must be documented using double-asterisk Javadoc/kernel-doc style blocks:
    ```c
    /**
     * function_name - Short summary of what this function does.
     * @param1: Description of parameter 1.
     *
     * Return: Description of return value.
     */
    ```
*   **Macro Guards**: Define guards at the end of conditional preprocessor directives:
    ```c
    #ifndef HEADER_GUARD_H
    #define HEADER_GUARD_H
    ...
    #endif /* HEADER_GUARD_H */
    ```

---

## 2. Rust Source Files
Rust source files use standard Cargo and Rust doc comment conventions:
*   **Module Documentation**: Every module/crate file must start with inner doc comments (`//!`) describing the module's role.
*   **Public API Documentation**: All public items (`pub fn`, `pub struct`, `pub enum`, `pub trait`) must be documented using outer doc comments (`///`).
*   **Implementation Comments**: Standard implementation comments within functions use the `//` format.
*   **No Inline Comments**: Move all comments to their own line above the code.

---

## 3. Assembly Files
Assembly code (`.asm`, `.inc`) uses semicolon comments:
*   **Syntax**: Begin comments with a semicolon followed by a space `; `.
*   **Line Placement**: Comments must occupy their own line above the target instructions. Avoid writing inline comments on the same line as CPU instructions.

---

## 4. Linting and Formatting Tools
To maintain formatting consistency, the repository includes configurations for code linters:
*   **Clang-Format (`.clang-format`)**: Standardizes spaces, alignments, and brackets for C code. Run it before committing C files.
*   **Clangd Config (`.clangd`)**: Outlines compile commands and include paths for C editor autocomplete.
*   **Rustfmt (`rustfmt.toml`)**: Standardizes Rust indentation and line wraps.
*   **Clippy (`.clippy.toml`)**: Enforces Rust code analysis checks (e.g. maximum function parameters and line width).

---

## 5. Language & Grammar Rules
To maintain professional open-source standards:
*   **English Language Only**: All code comments, docstrings, terminal output messages, commit messages, and documentation files must strictly be written in English with clean grammar.
*   **Zero Non-English Comments**: Non-English code comments, variable names, or prompt strings are strictly prohibited across all C, Rust, Assembly, and Markdown files.
