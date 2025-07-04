
ALTER TABLE auctions
ADD COLUMN IF NOT EXISTS highest_bidder_id UUID REFERENCES users(id) ON DELETE SET NULL;

ALTER TABLE auctions
ADD COLUMN IF NOT EXISTS current_highest_bid DECIMAL(12, 2);