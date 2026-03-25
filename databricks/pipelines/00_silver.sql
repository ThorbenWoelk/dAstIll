-- Databricks notebook source
-- Spark Declarative Pipeline: Silver layer
--
-- Reads from the Bronze table written by the backend (external to this pipeline).
-- Produces a temporary view: silver_app_events
--
-- The Bronze table is referenced via the `bronze_table` pipeline configuration key.
-- Set it in databricks.yml under resources.pipelines.analytics_transformations.configuration.
-- Default: analytics.bronze_app_events

-- COMMAND ----------

CREATE OR REFRESH PRIVATE MATERIALIZED VIEW silver_app_events
COMMENT "Typed, deduplicated app events. One row per event_id, earliest received_at wins. Private: not published to the catalog."
AS
WITH deduped AS (
  SELECT
    received_at,
    event_id,
    TRY_CAST(event_time AS TIMESTAMP)                                   AS event_ts,
    event_name,
    session_id,
    channel_id,
    video_id,
    summary_id,
    TRY_CAST(get_json_object(raw_json, '$.success')              AS BOOLEAN) AS success,
    TRY_CAST(get_json_object(raw_json, '$.acknowledged')         AS BOOLEAN) AS acknowledged,
    TRY_CAST(get_json_object(raw_json, '$.video_count')          AS INT)     AS video_count,
    get_json_object(raw_json, '$.from_mode')                               AS from_mode,
    get_json_object(raw_json, '$.to_mode')                                 AS to_mode,
    get_json_object(raw_json, '$.source')                                  AS highlight_source,
    TRY_CAST(get_json_object(raw_json, '$.read_time_ms')         AS BIGINT) AS read_time_ms,
    TRY_CAST(get_json_object(raw_json, '$.active_time_ms')       AS BIGINT) AS active_time_ms,
    TRY_CAST(get_json_object(raw_json, '$.max_scroll_depth_pct') AS DOUBLE) AS max_scroll_depth_pct,
    ROW_NUMBER() OVER (PARTITION BY event_id ORDER BY received_at ASC)     AS rn
  FROM ${bronze_table}
)
SELECT
  received_at,
  event_id,
  event_ts,
  event_name,
  session_id,
  channel_id,
  video_id,
  summary_id,
  success,
  acknowledged,
  video_count,
  from_mode,
  to_mode,
  highlight_source,
  read_time_ms,
  active_time_ms,
  max_scroll_depth_pct
FROM deduped
WHERE rn = 1;
