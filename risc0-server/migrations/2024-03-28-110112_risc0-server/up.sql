-- Your SQL goes here
CREATE TABLE IF NOT EXISTS vms (
  id SERIAL PRIMARY KEY,
  project_name VARCHAR NOT NULL,
  elf TEXT NOT NULL,
  image_id VARCHAR NOT NULL
);

CREATE TABLE proofs (
  id SERIAL PRIMARY KEY,
  project_id VARCHAR NOT NULL,
  task_id VARCHAR NOT NULL,
  client_id VARCHAR NOT NULL,
  sequencer_sign VARCHAR NOT NULL,
  image_id VARCHAR NOT NULL,
  datas_input VARCHAR NOT NULL,
  receipt_type VARCHAR NOT NULL,
  receipt TEXT,
  status VARCHAR NOT NULL,
  create_at TIMESTAMP NOT NULL DEFAULT now()
);