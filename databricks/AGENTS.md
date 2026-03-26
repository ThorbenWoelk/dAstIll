# Databricks (this repo)

Guidance for bundles, Lakeflow Spark Declarative Pipelines (SDP) SQL, and dashboards under `databricks/`.

For product UI, typography, colors, and icon conventions used by the web app, see the repo root [DESIGN.md](../DESIGN.md).

## SQL pipeline file naming

Use a single convention so library order in `databricks.yml` stays obvious and layers stay visible:

```
{index}_{content}_{layer}.sql
```

| Part | Rule |
| --- | --- |
| `index` | Two-digit sort order matching `resources.pipelines.*.libraries` (00, 01, …). |
| `content` | Short `snake_case` topic (e.g. `app_events`, `processing`, `engagement`). |
| `layer` | Medallion: `bronze`, `silver`, or `gold`. |

Examples (this repo): `00_app_events_silver.sql`, `01_processing_gold.sql`, `02_engagement_gold.sql`, `03_consumption_gold.sql`.

Files under `pipelines/` are **plain SQL sources**. Do not use Databricks notebook headers (`Databricks notebook source`, `COMMAND` cells). In `databricks.yml`, reference them with `libraries.file`, not `libraries.notebook`.

## Lakeflow SDP (official best practices, summarized)

Authoritative detail: [Best practices for Lakeflow Spark Declarative Pipelines](https://docs.databricks.com/en/ldp/best-practices.html).

- **Dataset types**: Use **streaming tables** for append-only ingestion and CDC-style feeds; **materialized views** for heavy transforms, aggregations, and dashboard-facing tables; **temporary views** for pipeline-only intermediates that should not materialize storage.
- **CDC**: Prefer declarative **`APPLY CHANGES INTO`** over hand-rolled `MERGE` when you need ordering, deduplication, out-of-order handling, and schema evolution on change feeds.
- **Data quality**: Use **expectations** (constraints) with an explicit violation policy: `WARN` (default), `DROP ROW`, or `FAIL UPDATE`, depending on whether bad rows must be kept, discarded, or block the run.
- **Parameters**: Avoid hardcoding catalog/schema (or other environment-specific names) in SQL. Use pipeline **configuration** keys and `${key}` substitution (see `bronze_table` in `databricks.yml`).
- **Pipeline mode**: Prefer **triggered** updates for scheduled or on-demand freshness; use **continuous** only when you need low latency and accept higher compute cost.
- **Targets**: Use **development** mode for fast iteration; **production** for cost-aware shutdown and retries (aligned with bundle `targets` in `databricks.yml`).
- **Layout**: For streaming tables, consider **liquid clustering** (`CLUSTER BY`) instead of relying only on static partitions; `CLUSTER BY AUTO` is an option when workload is unclear.
- **Operations**: Prefer **Databricks Asset Bundles** (this folder’s `databricks.yml`) for repeatable deploys and CI/CD.

## Unity Catalog

Use a single catalog + schema per target where possible; grant least privilege on published tables and views. Pipeline defaults and variables live in `databricks.yml` (`catalog`, `schema`, `bronze_schema`). See also [Unity Catalog best practices](https://docs.databricks.com/en/data-governance/unity-catalog/best-practices.html).

## This bundle

- **Library order** in `analytics_transformations.libraries` defines execution order; keep indices in filenames in sync.
- **`bronze_table`**: Full three-level name for the Bronze table the Silver SQL source reads; must match where the app writes events.
- **Dashboards**: `.lvdash.json` files reference warehouse IDs via bundle variables; do not embed secrets in repo files.
