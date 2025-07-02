-- Add migration for creating vehicles table
CREATE TABLE IF NOT EXISTS vehicles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    make TEXT NOT NULL,
    model TEXT NOT NULL,
    year INT NOT NULL,
    price DECIMAL(12, 2) NOT NULL,
    mileage INT,
    exterior_color TEXT,
    interior_color TEXT,
    engine TEXT,
    transmission TEXT,
    fuel_type TEXT,
    image_urls TEXT[],
    features TEXT[],
    description TEXT,
    status TEXT DEFAULT 'available',
    is_featured BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);
