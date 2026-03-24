#!/usr/bin/env bash
# Creates a sample SQLite database for the data app demo.
# Usage: bash create-sample-db.sh
set -euo pipefail

DB="data.db"
rm -f "$DB"

sqlite3 "$DB" <<'SQL'
CREATE TABLE products (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    price REAL NOT NULL,
    units_sold INTEGER NOT NULL,
    region TEXT NOT NULL
);

INSERT INTO products (name, category, price, units_sold, region) VALUES
('Widget A',    'Hardware',   29.99,  1240, 'DACH'),
('Widget B',    'Hardware',   49.99,   890, 'DACH'),
('Widget C',    'Hardware',   19.99,  2100, 'Nordics'),
('Service X',   'Consulting', 150.00,  340, 'DACH'),
('Service Y',   'Consulting', 200.00,  210, 'UK'),
('Service Z',   'Consulting', 175.00,  480, 'Nordics'),
('License Pro', 'Software',   99.00,  1560, 'DACH'),
('License Ent', 'Software',  299.00,   620, 'UK'),
('License Std', 'Software',   49.00,  3200, 'Nordics'),
('Training 1d', 'Education',  500.00,  180, 'DACH'),
('Training 3d', 'Education', 1200.00,   95, 'UK'),
('Training 5d', 'Education', 1800.00,   42, 'Nordics');

CREATE TABLE monthly_revenue (
    month TEXT NOT NULL,
    category TEXT NOT NULL,
    revenue REAL NOT NULL
);

INSERT INTO monthly_revenue (month, category, revenue) VALUES
('2025-01', 'Hardware',    48200),
('2025-02', 'Hardware',    52100),
('2025-03', 'Hardware',    61300),
('2025-04', 'Hardware',    55800),
('2025-05', 'Hardware',    63200),
('2025-06', 'Hardware',    71400),
('2025-01', 'Consulting', 112000),
('2025-02', 'Consulting', 98500),
('2025-03', 'Consulting', 124000),
('2025-04', 'Consulting', 108000),
('2025-05', 'Consulting', 131000),
('2025-06', 'Consulting', 119000),
('2025-01', 'Software',    85600),
('2025-02', 'Software',    92300),
('2025-03', 'Software',    88100),
('2025-04', 'Software',    96400),
('2025-05', 'Software',   102000),
('2025-06', 'Software',   110500),
('2025-01', 'Education',   36000),
('2025-02', 'Education',   42000),
('2025-03', 'Education',   54000),
('2025-04', 'Education',   48000),
('2025-05', 'Education',   39000),
('2025-06', 'Education',   57000);
SQL

echo "Created $DB ($(du -h "$DB" | cut -f1) with 12 products + 24 monthly revenue rows)"
