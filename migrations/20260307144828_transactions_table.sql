-- Add migration script here
CREATE TABLE if not exists transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_user UUID REFERENCES users(id),
    to_user UUID REFERENCES users(id),
    amount INTEGER NOT NULL,
    order_id UUID REFERENCES orders(id),
    created_at TIMESTAMP DEFAULT now()
);
