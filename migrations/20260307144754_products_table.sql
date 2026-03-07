-- Add migration script here
CREATE TABLE if not exists products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seller_id UUID REFERENCES users(id),
    name TEXT NOT NULL,
    description TEXT,
    price INTEGER NOT NULL,
    stock INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT now()
);
