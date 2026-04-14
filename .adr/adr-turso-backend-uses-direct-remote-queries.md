# ADR: Turso Backend Uses Direct Remote Queries Instead Of Embedded Replicas

## Status

Accepted

## Context

The backend ran on Cloud Run with a Turso embedded replica stored on ephemeral disk.
Each instance performed startup sync work and background sync refreshes.

That design consumed the Turso monthly sync quota quickly because Cloud Run instance
churn multiplied full or partial replica sync traffic. The backend workload did not
need offline reads, and the plan pressure came from sync volume rather than normal
row reads or writes.

## Decision

The backend should use direct remote Turso queries instead of embedded replicas.

Concretely:

- production and one-shot backend tools should create Turso connections with
  `libsql::Builder::new_remote(...)`
- the backend should stop creating local embedded replica files for Turso-backed
  runtime reads
- local fallback remains a plain local libSQL file when Turso is not configured

## Consequences

Positive:

- removes replica sync traffic from steady-state backend operation
- avoids cold-start replica rebuild cost on Cloud Run
- keeps the existing query layer intact with a smaller code change

Tradeoffs:

- all Turso-backed backend reads now depend on live remote database availability
- keyword-search traffic now counts against normal Turso read limits instead of
  being served from a local replica
- org or database read blocks from billing or quota state become immediately
  visible to the backend

## Directive

Do not reintroduce embedded replicas for the backend unless the deployment model
changes enough to make sync traffic predictable and affordable. If low-latency
local search becomes necessary again, redesign around explicit cost controls
before restoring replica-based reads.
