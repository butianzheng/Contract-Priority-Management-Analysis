-- ============================================
-- Migration 018: Add batch_id column to intervention_log
-- Fixes: "table intervention_log has no column named batch_id" error
-- ============================================

-- Add batch_id column to intervention_log table
-- This column links intervention records to batch_operations for batch adjustments
ALTER TABLE intervention_log ADD COLUMN batch_id INTEGER REFERENCES batch_operations(batch_id);

-- Create index for better query performance on batch_id lookups
CREATE INDEX IF NOT EXISTS idx_intervention_log_batch_id ON intervention_log(batch_id);
