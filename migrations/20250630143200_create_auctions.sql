-- Add migration for creating auctions table
-- You will need to run `sqlx migrate add add_highest_bidder_id_to_auctions`
-- and then copy this content into the new migration file.
-- Then run `sqlx migrate run`.
CREATE TABLE IF NOT EXISTS auctions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id UUID REFERENCES vehicles(id) ON DELETE CASCADE,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    starting_bid DECIMAL(12, 2) NOT NULL,
    current_highest_bid DECIMAL(12, 2),
    highest_bidder_id UUID REFERENCES users(id) ON DELETE SET NULL, -- ADDED THIS LINE
    status TEXT DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);