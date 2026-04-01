CREATE TABLE IF NOT EXISTS mbs_items (
  item_num INTEGER PRIMARY KEY,
  sub_item_num INTEGER,
  item_start_date TEXT,
  item_end_date TEXT,
  category TEXT,
  group_code TEXT,
  sub_group TEXT,
  sub_heading TEXT,
  item_type TEXT,
  fee_type TEXT,
  provider_type TEXT,
  schedule_fee REAL,
  benefit_75 REAL,
  benefit_85 REAL,
  benefit_100 REAL,
  derived_fee TEXT,
  description TEXT,
  description_start_date TEXT,
  emsn_cap TEXT,
  emsn_maximum_cap REAL,
  emsn_percentage_cap REAL,
  is_gst_free INTEGER NOT NULL DEFAULT 1,
  is_active INTEGER NOT NULL DEFAULT 1,
  imported_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_mbs_category ON mbs_items(category);
CREATE INDEX IF NOT EXISTS idx_mbs_active ON mbs_items(is_active);
CREATE INDEX IF NOT EXISTS idx_mbs_provider ON mbs_items(provider_type);
