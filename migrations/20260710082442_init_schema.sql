CREATE TABLE categories (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE expenses (
    id TEXT PRIMARY KEY NOT NULL,
    amount REAL NOT NULL,
    category_id TEXT NOT NULL REFERENCES categories (id),
    description TEXT NOT NULL,
    date TEXT NOT NULL
);
