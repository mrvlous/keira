# SPDX-License-Identifier: GPL-2.0-only

# Sphinx Documentation Build System Architecture
#
# Orchestrates building HTML pages, running link checks, verifying
# style guide compliance and cleaning documentation artifacts for Keira Kernel.

SHELL           := /bin/bash

# Include Linux-style configuration file if present
-include ../.config

# Verbose mode: set V=1 to display raw command executions
ifeq ($(V),1)
    Q           :=
else
    Q           := @
endif

# Toolchain executables & directories
SPHINXOPTS      ?= -W
SPHINXBUILD     ?= $(shell which sphinx-build 2>/dev/null || echo "../.venv/bin/sphinx-build")
SOURCEDIR       = .
BUILDDIR        = _build
PYTHON          ?= $(shell which python3 2>/dev/null || echo "../.venv/bin/python")
SPHINX_REQ      := sphinx/requirements.txt

# Uniform logging definitions matching kernel master build system
LOG_INFO        := printf "[INFO]  %s\n"
LOG_DONE        := printf "[DONE]  %s\n"
LOG_WARN        := printf "[WARN]  %s\n"
LOG_ERR         := printf "[ERR]   %s\n"
LOG_DOCS        := printf "  [SPHINX] %s\n"

# Phony targets declaration
.PHONY: all html htmldocs pdf pdfdocs latexpdf linkcheck lint check-style clean distclean mrproper info help venv

all: html ## Build Sphinx HTML documentation (default)

help: ## Display available Sphinx documentation build targets
	@printf "\nKeira Documentation Build System\n\n"
	@printf "  Usage: make -C docs <target> [SPHINXOPTS=opts] [V=1]\n\n"
	@printf "  Targets:\n"
	@grep -E '^[a-zA-Z0-9_ -]+:.*?## .*$$' $(firstword $(MAKEFILE_LIST)) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "    %-20s %s\n", $$1, $$2}'
	@printf "\n"

venv: ## Create virtual environment and install Sphinx dependencies from sphinx/requirements.txt
	@$(LOG_INFO) "Setting up Python virtual environment in ../.venv..."
	$(Q)$(PYTHON) -m venv ../.venv
	@$(LOG_INFO) "Installing Sphinx requirements from $(SPHINX_REQ)..."
	$(Q)../.venv/bin/pip install -r $(SPHINX_REQ)
	@$(LOG_DONE) "Documentation environment ready"

html htmldocs: ## Compile Sphinx reStructuredText sources into HTML pages (Linux standard)
	@$(LOG_INFO) "Compiling Sphinx HTML documentation..."
	$(Q)$(SPHINXBUILD) -b html "$(SOURCEDIR)" "$(BUILDDIR)/html" $(SPHINXOPTS) $(O)
	@$(LOG_DONE) "HTML documentation build complete: $(BUILDDIR)/html/index.html"

linkcheck: ## Verify integrity of external hyperlinks in documentation
	@$(LOG_INFO) "Checking external documentation hyperlinks..."
	$(Q)$(SPHINXBUILD) -b linkcheck "$(SOURCEDIR)" "$(BUILDDIR)/linkcheck" $(SPHINXOPTS) $(O)
	@$(LOG_DONE) "Link check complete"

latexpdf pdf pdfdocs: ## Compile PDF documentation via LaTeX (if pdflatex is installed)
	@$(LOG_INFO) "Compiling PDF documentation via Sphinx LaTeX builder..."
	$(Q)$(SPHINXBUILD) -b latex "$(SOURCEDIR)" "$(BUILDDIR)/latex" $(SPHINXOPTS) $(O)
	$(Q)$(MAKE) -C "$(BUILDDIR)/latex" all-pdf
	@$(LOG_DONE) "PDF documentation build complete: $(BUILDDIR)/latex/"

lint check-style: ## Verify all documentation files for SPDX and style compliance
	@$(LOG_INFO) "Running documentation style & SPDX linter..."
	$(Q)$(PYTHON) sphinx/check_docs.py
	@$(LOG_DONE) "Documentation style verification complete"

info: ## Display Sphinx toolchain environment and page metrics
	@printf "\nKeira Sphinx Documentation Build Environment\n\n"
	@printf "  Sphinx Build    : $(SPHINXBUILD)\n"
	@printf "  Python Binary   : $(PYTHON)\n"
	@printf "  Requirements    : $(SPHINX_REQ)\n"
	@printf "  Source Directory: $(SOURCEDIR)\n"
	@printf "  Build Output    : $(BUILDDIR)\n"
	@printf "  SPHINXOPTS      : $(SPHINXOPTS)\n"
	@printf "  Documentation   : %d .rst source files\n\n" $(shell find $(SOURCEDIR) -name "*.rst" ! -path "*/$(BUILDDIR)/*" | wc -l)

clean: ## Remove generated documentation build directory
	@$(LOG_INFO) "Cleaning documentation build artifacts..."
	$(Q)rm -rf "$(BUILDDIR)"
	@$(LOG_DONE) "Documentation clean complete"

distclean mrproper: clean ## Clean build artifacts, pycache and temporary files
	@$(LOG_INFO) "Cleaning documentation to pristine state..."
	$(Q)find . -type f \( -name "*~" -o -name "*.swp" -o -name "*.swo" -o -name "*.bak" -o -name "*.tmp" -o -name "*.pyc" \) -delete 2>/dev/null || true
	$(Q)find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	@$(LOG_DONE) "Documentation distclean complete"
