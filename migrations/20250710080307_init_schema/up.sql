-- users table
CREATE TABLE users (
  id          UUID        PRIMARY KEY,
  email       TEXT        NOT NULL UNIQUE,
  password_hash TEXT      NOT NULL,
  role        TEXT        NOT NULL DEFAULT 'user',
  status      TEXT        NOT NULL DEFAULT 'active',
  created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- vehicles table
CREATE TABLE vehicles (
  id          UUID        PRIMARY KEY,
  make        TEXT        NOT NULL,
  model       TEXT        NOT NULL,
  year        INT         NOT NULL,
  mileage     INT         NOT NULL,
  price       NUMERIC(12,2) NOT NULL,
  description TEXT,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- auctions table
CREATE TABLE auctions (
  id                    UUID        PRIMARY KEY,
  vehicle_id            UUID        NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
  start_time            TIMESTAMPTZ NOT NULL,
  end_time              TIMESTAMPTZ NOT NULL,
  starting_bid          NUMERIC(12,2) NOT NULL,
  current_highest_bid   NUMERIC(12,2),
  status                TEXT        NOT NULL,
  created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- bids table
CREATE TABLE bids (
  id          UUID        PRIMARY KEY,
  auction_id  UUID        NOT NULL REFERENCES auctions(id) ON DELETE CASCADE,
  amount      NUMERIC(12,2) NOT NULL,
  placed_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- inquiries table
CREATE TABLE inquiries (
  id          UUID        PRIMARY KEY,
  name        TEXT        NOT NULL,
  email       TEXT        NOT NULL,
  subject     TEXT        NOT NULL,
  message     TEXT        NOT NULL,
  status      TEXT        NOT NULL DEFAULT 'New',
  response    TEXT,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
