-- Audit logs table for OpenGP
-- Stores immutable audit trail of all changes to critical entities
-- This is an append-only table - NO UPDATE or DELETE operations allowed

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY,
    
    -- Entity information
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    
    -- Action details (JSON serialized AuditAction enum)
    action TEXT NOT NULL,
    
    -- Previous and new values (JSON serialized)
    old_value TEXT,
    new_value TEXT,
    
    -- Audit metadata
    changed_by UUID,
    changed_at TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Indexes for common audit queries
CREATE INDEX idx_audit_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_user ON audit_logs(changed_by);
CREATE INDEX idx_audit_time ON audit_logs(changed_at);
CREATE INDEX idx_audit_entity_time ON audit_logs(entity_type, entity_id, changed_at);
