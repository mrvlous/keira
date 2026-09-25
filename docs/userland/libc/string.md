<!-- SPDX-License-Identifier: GPL-2.0-only -->

# String & Memory Operations (`userland/lib/string/`)

High-performance memory manipulation and string inspection routines.

---

## Core Operations

* **Memory Copy & Set**:
  * `memcpy(void *dst, const void *src, size_t n)`: Fast byte copy with word alignment optimizations.
  * `memmove(void *dst, const void *src, size_t n)`: Overlap-safe memory copying.
  * `memset(void *dst, int val, size_t n)`: Constant-fill memory routine.
  * `memcmp(const void *s1, const void *s2, size_t n)`: Lexicographical byte comparison.
* **String Operations**:
  * `strlen(const char *s)`: Returns length of null-terminated string.
  * `strcpy`, `strncpy`: Copies source string into destination buffer.
  * `strcmp`, `strncmp`: Lexicographical string comparison.
  * `strchr`, `strrchr`: Locates first or last occurrence of character in string.
  * `strstr(const char *haystack, const char *needle)`: Substring search.
  * `strtok(char *str, const char *delim)`: Tokenizes string using delimiters.
