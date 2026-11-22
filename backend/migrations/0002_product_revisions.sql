CREATE SCHEMA engineering;

CREATE TABLE engineering.items (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    part_number text NOT NULL UNIQUE CHECK (length(btrim(part_number)) BETWEEN 1 AND 80)
);

CREATE TABLE engineering.item_revisions (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    item_id bigint NOT NULL REFERENCES engineering.items(id),
    revision text NOT NULL CHECK (length(btrim(revision)) BETWEEN 1 AND 40),
    description text NOT NULL CHECK (length(btrim(description)) BETWEEN 1 AND 250),
    CONSTRAINT item_revisions_item_revision_key UNIQUE (item_id, revision)
);

ALTER TABLE manufacturing.work_orders
    ADD COLUMN item_revision_id bigint REFERENCES engineering.item_revisions(id);
