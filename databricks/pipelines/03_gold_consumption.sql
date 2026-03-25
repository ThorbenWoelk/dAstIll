-- Databricks notebook source
-- Spark Declarative Pipeline: Gold - Summary Consumption
--
-- One row per summary reading session. Prefers summary_closed events for accurate
-- read_time_ms; falls back to the latest summary_heartbeat for sessions where close
-- delivery was not reliable (e.g., hard tab close on mobile).
--
-- Primary ranking metric:
--   engagement_score = active_time_ms * (1 + scroll_depth / 100)
-- Weights credible reading time by scroll coverage. Use this over raw read_time_ms,
-- which inflates with idle tabs and backgrounded sessions.

-- COMMAND ----------

CREATE OR REFRESH MATERIALIZED VIEW gold_summary_consumption
COMMENT "Summary reading attention: one row per session with active_time_ms, scroll depth, and engagement score."
AS
WITH closed_sessions AS (
  SELECT
    summary_id,
    video_id,
    channel_id,
    session_id,
    DATE(event_ts)       AS event_date,
    read_time_ms,
    active_time_ms,
    max_scroll_depth_pct,
    TRUE                 AS has_close_event
  FROM silver_app_events
  WHERE event_name = 'summary_closed'
    AND active_time_ms IS NOT NULL
    AND read_time_ms   IS NOT NULL
),
heartbeat_sessions AS (
  SELECT
    summary_id,
    video_id,
    channel_id,
    session_id,
    DATE(MAX(event_ts))       AS event_date,
    CAST(NULL AS BIGINT)      AS read_time_ms,
    MAX(active_time_ms)       AS active_time_ms,
    MAX(max_scroll_depth_pct) AS max_scroll_depth_pct,
    FALSE                     AS has_close_event
  FROM silver_app_events
  WHERE event_name = 'summary_heartbeat'
  GROUP BY summary_id, video_id, channel_id, session_id
),
combined AS (
  SELECT * FROM closed_sessions

  UNION ALL

  SELECT h.*
  FROM heartbeat_sessions h
  WHERE NOT EXISTS (
    SELECT 1
    FROM closed_sessions c
    WHERE c.summary_id = h.summary_id
      AND c.session_id = h.session_id
  )
)
SELECT
  summary_id,
  video_id,
  channel_id,
  session_id,
  event_date,
  active_time_ms,
  read_time_ms,
  max_scroll_depth_pct,
  has_close_event,
  ROUND(
    active_time_ms * (1.0 + COALESCE(max_scroll_depth_pct, 0) / 100.0)
  ) AS engagement_score
FROM combined
WHERE active_time_ms > 0;
