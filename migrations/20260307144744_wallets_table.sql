-- Add migration script here
CREATE TABLE if not exists wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE REFERENCES users(id),
    balance INTEGER DEFAULT 0
);
