-- Add highest_bidder_id to auctions
ALTER TABLE auctions
ADD COLUMN highest_bidder_id UUID
    REFERENCES users(id)
    ON DELETE SET NULL;
