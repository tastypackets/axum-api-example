CREATE TABLE users (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    private_notes TEXT NOT NULL
);

INSERT INTO users (username, display_name, private_notes) VALUES
    ('admin', 'Example Admin', 'Private admin account notes'),
    ('alice', 'Alice Chen', 'Private Alice account notes'),
    ('bob', 'Bob Rivera', 'Private Bob account notes');
