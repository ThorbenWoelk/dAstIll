-- Databricks notebook source
-- Spark Declarative Pipeline: Gold - Processing Efficiency
--
-- Pairs transcript_ensure_requested with transcript_ensure_completed within the
-- same (video, channel, session). One row per processing attempt.
-- Null processing_duration_ms means the request is still pending or was dropped.

-- COMMAND ----------

CREATE OR REFRESH MATERIALIZED VIEW gold_processing_efficiency
COMMENT "Content processing efficiency: one row per transcript processing attempt with duration and success flag."
AS
WITH requested AS (
  SELECT
    video_id,
    channel_id,
    session_id,
    event_ts       AS requested_at,
    DATE(event_ts) AS event_date
  FROM silver_app_events
  WHERE event_name = 'transcript_ensure_requested'
),
completed AS (
  SELECT
    video_id,
    channel_id,
    session_id,
    event_ts AS completed_at,
    success
  FROM silver_app_events
  WHERE event_name = 'transcript_ensure_completed'
)
SELECT
  r.video_id,
  r.channel_id,
  r.session_id,
  r.event_date,
  r.requested_at,
  c.completed_at,
  COALESCE(c.success, FALSE) AS success,
  CASE
    WHEN c.completed_at IS NOT NULL
    THEN TIMESTAMPDIFF(MILLISECOND, r.requested_at, c.completed_at)
  END AS processing_duration_ms
FROM requested r
LEFT JOIN completed c
  ON  r.video_id   = c.video_id
  AND r.channel_id = c.channel_id
  AND r.session_id = c.session_id
  AND c.completed_at >= r.requested_at
QUALIFY ROW_NUMBER() OVER (
  PARTITION BY r.video_id, r.channel_id, r.session_id
  ORDER BY r.requested_at DESC
) = 1;
