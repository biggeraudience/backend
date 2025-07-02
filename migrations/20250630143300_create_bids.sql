-- Create bids table
CREATE TABLE IF NOT EXISTS bids (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    auction_id UUID NOT NULL REFERENCES auctions(id) ON DELETE CASCADE,
    bidder_id UUID REFERENCES users(id) ON DELETE SET NULL,
    bid_amount NUMERIC(12, 2) NOT NULL,
    bid_time TIMESTAMPTZ NOT NULL DEFAULT now()
);
