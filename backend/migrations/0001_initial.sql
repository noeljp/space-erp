CREATE SCHEMA core;
CREATE SCHEMA manufacturing;

CREATE TABLE core.users (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    username text NOT NULL UNIQUE,
    display_name text NOT NULL,
    password_hash text NOT NULL,
    role text NOT NULL CHECK (role IN ('operator', 'supervisor')),
    active boolean NOT NULL DEFAULT true
);

CREATE TABLE core.login_sessions (
    token_hash bytea PRIMARY KEY,
    user_id bigint NOT NULL REFERENCES core.users(id),
    expires_at timestamptz NOT NULL
);

CREATE TABLE core.audit_events (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    actor_id bigint NOT NULL REFERENCES core.users(id),
    action text NOT NULL,
    entity_type text NOT NULL,
    entity_id bigint NOT NULL,
    occurred_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    payload jsonb NOT NULL
);

CREATE TABLE manufacturing.work_orders (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    number text NOT NULL UNIQUE CHECK (length(btrim(number)) BETWEEN 1 AND 80),
    description text NOT NULL CHECK (length(btrim(description)) BETWEEN 1 AND 250),
    quantity integer NOT NULL CHECK (quantity > 0),
    status text NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed')),
    created_at timestamptz NOT NULL DEFAULT clock_timestamp()
);

CREATE TABLE manufacturing.work_sessions (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    operator_id bigint NOT NULL REFERENCES core.users(id),
    work_order_id bigint NOT NULL REFERENCES manufacturing.work_orders(id),
    started_at timestamptz NOT NULL DEFAULT clock_timestamp(),
    ended_at timestamptz,
    CHECK (ended_at IS NULL OR ended_at > started_at)
);

CREATE UNIQUE INDEX one_active_session_per_operator
    ON manufacturing.work_sessions (operator_id) WHERE ended_at IS NULL;

CREATE INDEX work_sessions_by_order ON manufacturing.work_sessions (work_order_id);
