CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    action TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    changed_by UUID NOT NULL,
    changed_at TIMESTAMPTZ NOT NULL,
    FOREIGN KEY (changed_by) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_logs(changed_by);
CREATE INDEX IF NOT EXISTS idx_audit_time ON audit_logs(changed_at);
CREATE INDEX IF NOT EXISTS idx_audit_entity_time ON audit_logs(entity_type, entity_id, changed_at);
