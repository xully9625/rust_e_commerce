-- Add migration script here
CREATE TABLE if not exists orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    buyer_id UUID REFERENCES users(id),
    total_price INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT now()
);
