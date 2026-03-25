-- Gold: engagement progression (acknowledgment, highlights, opens).
--
-- One row per (video, channel). Tiers (high to low): acknowledged_and_highlighted,
-- acknowledged, opened, unseen.

CREATE OR REFRESH MATERIALIZED VIEW gold_engagement_progression
COMMENT "Acknowledgment and highlights progression: one row per (video, channel) with engagement tier."
AS
WITH latest_ack AS (
  SELECT
    video_id,
    channel_id,
    MAX(CASE WHEN acknowledged = TRUE THEN 1 ELSE 0 END) = 1 AS is_acknowledged,
    MIN(CASE WHEN acknowledged = TRUE THEN event_ts END)      AS first_acknowledged_at,
    COUNT(*)                                                   AS ack_event_count
  FROM silver_app_events
  WHERE event_name = 'video_acknowledged_changed'
  GROUP BY video_id, channel_id
),
highlights AS (
  SELECT
    video_id,
    channel_id,
    COUNT(*)                      AS highlight_count,
    MIN(event_ts)                 AS first_highlight_at,
    COLLECT_SET(highlight_source) AS highlight_sources
  FROM silver_app_events
  WHERE event_name = 'highlight_created'
  GROUP BY video_id, channel_id
),
video_opens AS (
  SELECT
    video_id,
    channel_id,
    COUNT(*)      AS open_count,
    MIN(event_ts) AS first_opened_at
  FROM silver_app_events
  WHERE event_name = 'video_opened'
  GROUP BY video_id, channel_id
)
SELECT
  COALESCE(a.video_id,   h.video_id,   v.video_id)   AS video_id,
  COALESCE(a.channel_id, h.channel_id, v.channel_id) AS channel_id,
  COALESCE(a.is_acknowledged, FALSE)                  AS is_acknowledged,
  a.first_acknowledged_at,
  a.ack_event_count,
  COALESCE(h.highlight_count, 0)                      AS highlight_count,
  h.first_highlight_at,
  h.highlight_sources,
  COALESCE(v.open_count, 0)                           AS open_count,
  v.first_opened_at,
  CASE
    WHEN COALESCE(a.is_acknowledged, FALSE)
      AND COALESCE(h.highlight_count, 0) > 0 THEN 'acknowledged_and_highlighted'
    WHEN COALESCE(a.is_acknowledged, FALSE)  THEN 'acknowledged'
    WHEN COALESCE(v.open_count, 0) > 0       THEN 'opened'
    ELSE 'unseen'
  END AS engagement_tier
FROM latest_ack a
FULL OUTER JOIN highlights h
  ON  a.video_id   = h.video_id
  AND a.channel_id = h.channel_id
FULL OUTER JOIN video_opens v
  ON  COALESCE(a.video_id,   h.video_id)   = v.video_id
  AND COALESCE(a.channel_id, h.channel_id) = v.channel_id;
