---
description: List source-quality windows for one scored radar run
argument-hint: <run_id>
---

# Radar Source Quality

The user invoked this command with: $ARGUMENTS

Use `radar_source_quality` with the provided `run_id`. List the durable
source-quality windows written for that specific radar run after scoring. Treat
this as telemetry over already-ingested radar items and score overlays, not as
proof of scheduled operation, model scoring, or delivery.

If no rows exist, report that no source-quality windows have been written yet
instead of implying quality tracking is operational.
