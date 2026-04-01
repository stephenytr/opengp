CREATE TABLE IF NOT EXISTS invoices (
    id TEXT PRIMARY KEY,
    invoice_number TEXT NOT NULL,
    patient_id TEXT NOT NULL,
    practitioner_id TEXT NOT NULL,
    consultation_id TEXT,
    billing_type TEXT NOT NULL,
    status TEXT NOT NULL,
    issue_date TEXT NOT NULL,
    due_date TEXT,
    subtotal REAL NOT NULL,
    gst_amount REAL NOT NULL,
    total_amount REAL NOT NULL,
    amount_paid REAL NOT NULL,
    amount_outstanding REAL NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (practitioner_id) REFERENCES users(id),
    FOREIGN KEY (consultation_id) REFERENCES consultations(id)
);

CREATE TABLE IF NOT EXISTS invoice_items (
    id TEXT PRIMARY KEY,
    invoice_id TEXT NOT NULL,
    description TEXT NOT NULL,
    item_code TEXT,
    quantity INTEGER NOT NULL,
    unit_price REAL NOT NULL,
    amount REAL NOT NULL,
    is_gst_free INTEGER NOT NULL CHECK (is_gst_free IN (0, 1)),
    created_at TEXT NOT NULL,
    FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS payments (
    id TEXT PRIMARY KEY,
    invoice_id TEXT NOT NULL,
    patient_id TEXT NOT NULL,
    amount REAL NOT NULL,
    payment_method TEXT NOT NULL,
    payment_date TEXT NOT NULL,
    reference TEXT,
    notes TEXT,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (invoice_id) REFERENCES invoices(id),
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS medicare_claims (
    id TEXT PRIMARY KEY,
    invoice_id TEXT,
    patient_id TEXT NOT NULL,
    practitioner_id TEXT NOT NULL,
    claim_type TEXT NOT NULL,
    status TEXT NOT NULL,
    service_date TEXT NOT NULL,
    total_claimed REAL NOT NULL,
    total_benefit REAL NOT NULL,
    reference_number TEXT,
    submitted_at TEXT,
    processed_at TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (invoice_id) REFERENCES invoices(id),
    FOREIGN KEY (patient_id) REFERENCES patients(id),
    FOREIGN KEY (practitioner_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_invoices_patient ON invoices(patient_id);
CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);
CREATE INDEX IF NOT EXISTS idx_invoice_items_invoice ON invoice_items(invoice_id);
CREATE INDEX IF NOT EXISTS idx_payments_invoice ON payments(invoice_id);
CREATE INDEX IF NOT EXISTS idx_payments_patient ON payments(patient_id);
CREATE INDEX IF NOT EXISTS idx_claims_patient ON medicare_claims(patient_id);
CREATE INDEX IF NOT EXISTS idx_claims_status ON medicare_claims(status);
