CREATE TABLE IF NOT EXISTS contracts_ttl (
  contract_id VARCHAR PRIMARY KEY NOT NULL,
  automatic_bump BOOLEAN NOT NULL,
  live_until_ttl  INT NOT NULL
);