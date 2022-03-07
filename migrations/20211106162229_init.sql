CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- CUSTOM TYPES
CREATE TYPE node_status AS ENUM ('poweron', 'poweroff', 'rebooting');
CREATE TYPE operation_type AS ENUM ('poweron', 'poweroff', 'reboot');

-- TABLE: clusters
CREATE TABLE clusters
(
    id uuid DEFAULT uuid_generate_v1() NOT NULL PRIMARY KEY,
    name text NOT NULL,
    created_at timestamp with time zone default CURRENT_TIMESTAMP,
    updated_at timestamp with time zone
);

CREATE UNIQUE INDEX cluster_name ON clusters (name);

-- TABLE: nodes

CREATE TABLE nodes
(
    id uuid DEFAULT uuid_generate_v1() NOT NULL PRIMARY KEY,
    name text NOT NULL,
    cluster_id uuid NOT NULL CONSTRAINT nodes_clusters_id_fk
            REFERENCES clusters
            ON DELETE CASCADE,
    status node_status,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone
);

CREATE UNIQUE INDEX node_name ON nodes (name);

-- TABLE: operations

CREATE TABLE operations
(
    id uuid DEFAULT uuid_generate_v1() NOT NULL PRIMARY KEY,
    operation_type operation_type NOT NULL,
    node_id uuid NOT NULL CONSTRAINT operations_nodes_id_fk
            REFERENCES nodes
            ON DELETE CASCADE,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone
);
