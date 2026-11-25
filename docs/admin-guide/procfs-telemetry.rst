.. SPDX-License-Identifier: GPL-2.0-only


System Telemetry Nodes
======================

* ``/proc/uptime``: Seconds since kernel boot.
* ``/proc/meminfo``: RAM total, used, free and heap statistics.
* ``/proc/buddyinfo``: Binary buddy page order fragmentation across physical memory zones.
* ``/proc/slabinfo``: Slab object caches, active objects, tunables and slab counts.
* ``/proc/vmstat``: Virtual memory counters, page fault events and TLB metrics.
* ``/proc/daemons``: Background Ring 0 daemon operational telemetry (``[kthreadd]``, ``[kswapd0]``, ``[kflushd]``).
* ``/proc/waitqueue``: Registered kernel wait queues, active waiter counts and wakeup telemetry.
* ``/proc/workqueues``: Deferred kernel workqueue pools (``system_wq``, ``events_unbound``) and ``[kworker/0:0]`` execution metrics.
* ``/proc/cpuinfo``: CPU model, core topology and feature flags.
