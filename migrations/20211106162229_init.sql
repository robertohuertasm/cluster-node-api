CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE node_status AS ENUM ('poweron', 'poweroff', 'rebooting');

CREATE TABLE nodes
(
    id uuid DEFAULT uuid_generate_v1() NOT NULL CONSTRAINT nodes_pkey PRIMARY KEY,
    name text NOT NULL,
    cluster_id uuid NOT NULL,
    status node_status,
    created_at timestamp with time zone default CURRENT_TIMESTAMP,
    updated_at timestamp with time zone
);

CREATE UNIQUE INDEX node_name ON nodes (name);

CREATE TABLE clusters
(
    id uuid DEFAULT uuid_generate_v1() NOT NULL CONSTRAINT cluster_pkey PRIMARY KEY,
    name text NOT NULL,
    created_at timestamp with time zone default CURRENT_TIMESTAMP,
    updated_at timestamp with time zone
);

CREATE UNIQUE INDEX cluster_name ON clusters (name);
