-- Migration 005: Add Missing EXIF Fields
-- Add exposure_bias, white_balance, and rename flash_used to flash for consistency
-- Created: February 2025

-- Add missing EXIF fields to images table
ALTER TABLE images ADD COLUMN exposure_bias REAL;
ALTER TABLE images ADD COLUMN white_balance TEXT;

-- Note: SQLite doesn't support renaming columns directly in older versions
-- So we'll add a new 'flash' column and copy data from flash_used if it exists
-- Then we can deprecate flash_used in a future migration if needed
ALTER TABLE images ADD COLUMN flash TEXT;

-- Copy flash_used boolean to flash text field (if flash_used has data)
-- Convert boolean to text: 1 -> "Yes", 0 -> "No", NULL -> NULL
UPDATE images
SET flash = CASE
    WHEN flash_used = 1 THEN 'Yes'
    WHEN flash_used = 0 THEN 'No'
    ELSE NULL
END
WHERE flash_used IS NOT NULL;

-- End of migration 005
