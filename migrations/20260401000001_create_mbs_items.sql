CREATE TABLE IF NOT EXISTS mbs_items (
  item_num INTEGER PRIMARY KEY,
  sub_item_num INTEGER,
  item_start_date DATE,
  item_end_date DATE,
  category TEXT,
  group_code TEXT,
  sub_group TEXT,
  sub_heading TEXT,
  item_type TEXT,
  fee_type TEXT,
  provider_type TEXT,
  schedule_fee DOUBLE PRECISION,
  benefit_75 DOUBLE PRECISION,
  benefit_85 DOUBLE PRECISION,
  benefit_100 DOUBLE PRECISION,
  derived_fee TEXT,
  description TEXT,
  description_start_date DATE,
  emsn_cap TEXT,
  emsn_maximum_cap DOUBLE PRECISION,
  emsn_percentage_cap DOUBLE PRECISION,
  is_gst_free BOOLEAN NOT NULL DEFAULT TRUE,
  is_active BOOLEAN NOT NULL DEFAULT TRUE,
  imported_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_mbs_category ON mbs_items(category);
CREATE INDEX IF NOT EXISTS idx_mbs_active ON mbs_items(is_active);
CREATE INDEX IF NOT EXISTS idx_mbs_provider ON mbs_items(provider_type);
