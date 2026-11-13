-- OPC: Fix orphaned payments.invoice_id
-- Bug 5: 24 records reference non-existent invoices
-- This sets invoice_id to NULL for orphaned references

-- Preview: see which records will be affected
-- SELECT id, invoice_id, payment_type, amount
-- FROM payments
-- WHERE invoice_id IS NOT NULL
--   AND invoice_id NOT IN (SELECT id FROM invoices);

-- Fix: set orphaned invoice_id to NULL
UPDATE payments
SET invoice_id = NULL,
    updated_at = datetime('now')
WHERE invoice_id IS NOT NULL
  AND invoice_id NOT IN (SELECT id FROM invoices);
