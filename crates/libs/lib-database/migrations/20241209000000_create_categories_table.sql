-- Create categories table
CREATE TABLE IF NOT EXISTS categories (
    id TEXT PRIMARY KEY NOT NULL,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    url_slug TEXT,
    category_type TEXT NOT NULL,
    color TEXT,
    icon TEXT,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create index on code for faster lookups
CREATE INDEX IF NOT EXISTS idx_categories_code ON categories(code);

-- Create index on is_active for filtering
CREATE INDEX IF NOT EXISTS idx_categories_active ON categories(is_active);
