# SPDX-License-Identifier: GPL-2.0-only

"""Configuration helper and dynamic version extractor for Keira Kernel docs."""

import re
from pathlib import Path


def get_kernel_version() -> str:
    """Extract kernel release version from root Cargo.toml or fallback to 0.8.0."""
    try:
        repo_root = Path(__file__).resolve().parent.parent.parent
        cargo_path = repo_root / "Cargo.toml"
        if cargo_path.exists():
            content = cargo_path.read_text(encoding="utf-8")
            match = re.search(r'version\s*=\s*"([^"]+)"', content)
            if match:
                return match.group(1)
    except Exception:
        pass
    return "0.8.0"


def setup(app):
    """Sphinx extension setup hook."""
    return {
        "version": get_kernel_version(),
        "parallel_read_safe": True,
        "parallel_write_safe": True,
    }

