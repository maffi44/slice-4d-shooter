# Context: Frame server TCP messages and bound reads

[Current task](server-tcp-framing.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 7. Author requested an ordinary-priority issue (2026-11-16). This is inherited server behavior rather than a new world-isolation regression. Existing 300 ms matchmaking-side timeouts do not establish message boundaries or protect every read.

Related context: [authoritative server and transport](../network-authoritative-server-and-transport/). Fixing framing need not wait for a transport replacement.
