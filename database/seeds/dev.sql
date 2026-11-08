-- Optional development sample catalog. Run with psql after migrations.
INSERT INTO products (sku, name, category, unit, selling_price) VALUES
  ('BAK-001', 'Baguette', 'Bread', 'piece', 300),
  ('BAK-002', 'Croissant', 'Pastry', 'piece', 500),
  ('BAK-003', 'Chocolate cake slice', 'Cake', 'piece', 1500)
ON CONFLICT (sku) DO NOTHING;
