-- A receipt must always increase inventory.
-- Other movement types retain their existing workflow-specific sign rules.
ALTER TABLE inventory_movements
ADD CONSTRAINT inventory_receipt_positive_quantity
CHECK (kind <> 'receipt' OR quantity_delta > 0);