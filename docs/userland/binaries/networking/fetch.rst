.. SPDX-License-Identifier: GPL-2.0-only


fetch.elf Freestanding HTTP Client Utility
==========================================

The ``fetch.elf`` executable (``/bin/fetch.elf``, ``/bin/fetch`` and symbolic alias ``/bin/curl``) is a freestanding Ring 3 network client that retrieves resources and web APIs over HTTP from local and external network endpoints.

1. Architectural Purpose
========================

In accordance with UNIX purity and the Keira v0.8.0 pure kernel slimming roadmap, high-level network client operations are decoupled from the Ring 0 supervisor console and implemented as freestanding unprivileged Ring 3 binaries:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged Ring 3, requesting network resources exclusively through the kernel POSIX socket system call boundary.
2. **Real-Time Chunk Streaming**: Streams incoming HTTP response segments directly to standard output as network frames arrive without intermediate buffering or data dumping.
3. **Clean Payload Output**: Suppresses HTTP response headers, status lines and debug telemetry from standard output by default, matching Linux ``curl`` behavior for automated pipeline processing.
4. **Dynamic Domain Name Resolution**: Uses standard POSIX ``gethostbyname()`` backed by `/etc/hosts` and in-kernel DNS resolver system call (Vector 110, ``SYS_DNS_RESOLVE``) to resolve domain names dynamically.
5. **Dual Binary Identity**: Fully accessible under both ``fetch`` and ``curl`` command names.

2. Command-Line Options
=======================

.. list-table::
   :header-rows: 1
   :widths: 15 15 70

   * - Option
     - Long Option
     - Description
   * - ``-i``
     - ``--include``
     - Includes HTTP response status line and headers in standard output before payload
   * - ``-I``
     - ``--head``
     - Displays HTTP response status line and headers only, closing connection immediately
   * - ``-o, -O <file>``
     - *N/A*
     - Writes received payload to specified local file instead of standard output
   * - ``-s``
     - ``--silent``
     - Silent mode; suppresses error and diagnostic telemetry
   * - ``-v``
     - ``--verbose``
     - Displays connection and HTTP header telemetry on stderr without polluting stdout
   * - ``-h``
     - ``--help``
     - Displays usage reference and exits with status 0

3. Usage Examples
=================

Fetching Payloads to Standard Output
------------------------------------

.. code-block:: bash

   # Retrieve a remote API endpoint directly to stdout (clean payload only):
   keira:/# fetch http://api.example.com/data

   # Retrieve via curl alias:
   keira:/# curl http://httpbin.org/get

Writing Payloads to File
------------------------

.. code-block:: bash

   # Save remote payload to local disk file:
   keira:/# curl -o /tmp/payload.json http://httpbin.org/get

   # Inspect saved payload with cat:
   keira:/# cat /tmp/payload.json

Inspecting HTTP Response Headers
--------------------------------

.. code-block:: bash

   # Display only status line and headers:
   keira:/# curl -I http://example.com/

   # Include headers with payload:
   keira:/# curl -i http://example.com/

Verbose Network Diagnostics
---------------------------

.. code-block:: bash

   # Verbose transfer diagnostics directed to stderr:
   keira:/# curl -v http://example.com/

4. Technical Specifications
===========================

- **Source Path**: ``userland/netutils/fetch.c``
- **Binary Locations**: ``/bin/fetch.elf``, ``/bin/fetch``, ``/bin/curl.elf`` and ``/bin/curl``
- **Privilege Level**: Ring 3 unprivileged
- **Dependencies**: Freestanding ``libc.a`` (``<arpa/inet.h>``, ``<netdb.h>``, ``<sys/socket.h>``, ``<fcntl.h>``, ``<stdio.h>``, ``<stdlib.h>``, ``<string.h>`` and ``<unistd.h>``)
- **System Calls**: ``sys_socket`` (Vector 24), ``sys_connect`` (Vector 25), ``sys_write`` (Vector 8), ``sys_read`` (Vector 7), ``sys_open`` (Vector 6), ``sys_close`` (Vector 9), ``sys_dns_resolve`` (Vector 110) and ``sys_exit`` (Vector 2)
- **Streaming Buffer**: 1024-byte chunk buffer with real-time socket draining
