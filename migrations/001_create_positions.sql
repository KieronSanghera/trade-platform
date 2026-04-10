CREATE TABLE positions (
    user_id   TEXT    NOT NULL,
    asset     TEXT    NOT NULL,
    quantity  NUMERIC NOT NULL,
    avg_price NUMERIC NOT NULL,
    PRIMARY KEY (user_id, asset)
);