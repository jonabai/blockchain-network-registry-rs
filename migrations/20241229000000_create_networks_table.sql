-- Create UUID extension if not exists
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create networks table
CREATE TABLE IF NOT EXISTS networks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    chain_id INTEGER NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    rpc_url VARCHAR(500) NOT NULL,
    other_rpc_urls JSONB NOT NULL DEFAULT '[]',
    test_net BOOLEAN NOT NULL DEFAULT false,
    block_explorer_url VARCHAR(500) NOT NULL,
    fee_multiplier DECIMAL(10,4) NOT NULL DEFAULT 1.0,
    gas_limit_multiplier DECIMAL(10,4) NOT NULL DEFAULT 1.0,
    active BOOLEAN NOT NULL DEFAULT true,
    default_signer_address VARCHAR(42) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_networks_chain_id ON networks(chain_id);
CREATE INDEX IF NOT EXISTS idx_networks_active ON networks(active);
CREATE INDEX IF NOT EXISTS idx_networks_name ON networks(name);

-- Add constraint for Ethereum address format
ALTER TABLE networks ADD CONSTRAINT chk_default_signer_address
    CHECK (default_signer_address ~ '^0x[a-fA-F0-9]{40}$');

-- Add constraint for positive multipliers
ALTER TABLE networks ADD CONSTRAINT chk_fee_multiplier_positive
    CHECK (fee_multiplier >= 0);
ALTER TABLE networks ADD CONSTRAINT chk_gas_limit_multiplier_positive
    CHECK (gas_limit_multiplier >= 0);

-- Add constraint for positive chain_id
ALTER TABLE networks ADD CONSTRAINT chk_chain_id_positive
    CHECK (chain_id >= 1);
