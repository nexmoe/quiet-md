-- One row per (UTC day, metric, dimensions). Every statistic lands here: a new kind
-- of data is a new metric name, not a new table. `dims` is a JSON object with sorted
-- keys (written by the collector), so the same breakdown always has the same key.
CREATE TABLE metrics (
  day TEXT NOT NULL,                -- YYYY-MM-DD, UTC
  metric TEXT NOT NULL,             -- e.g. cdn.downloads, update.checks, github.downloads
  dims TEXT NOT NULL DEFAULT '{}',  -- e.g. {"channel":"cdn","country":"CN","platform":"macos-arm64","version":"latest"}
  value REAL NOT NULL,
  source TEXT NOT NULL,             -- where the number came from: cloudflare, github
  collected_at TEXT NOT NULL,
  PRIMARY KEY (day, metric, dims)
);
CREATE INDEX metrics_metric_day ON metrics (metric, day);

-- One row per collector run and source, for checking that the cron keeps up.
CREATE TABLE runs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  started_at TEXT NOT NULL,
  source TEXT NOT NULL,
  day TEXT,
  rows INTEGER NOT NULL DEFAULT 0,
  error TEXT
);

-- Downloads per day and channel, all sources together: the headline number.
-- cdn.downloads counts distinct IPs per file and day (a resumed or segmented download
-- is one), github.downloads is the day's increase of GitHub's asset counter.
CREATE VIEW downloads_daily AS
SELECT day,
       json_extract(dims, '$.platform') AS platform,
       CASE WHEN metric = 'github.downloads' THEN 'github' ELSE json_extract(dims, '$.channel') END AS channel,
       SUM(value) AS downloads
FROM metrics
WHERE metric IN ('cdn.downloads', 'github.downloads')
GROUP BY day, platform, channel;
