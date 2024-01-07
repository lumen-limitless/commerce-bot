#!/bin/bash

DATABASE_URL="database.db"

# Create tables
sqlite3 $DATABASE_URL <<EOF
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS products (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    image TEXT NOT NULL,
    price INTEGER NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS carts (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id),
    UNIQUE (user_id)
);

CREATE TABLE IF NOT EXISTS cart_items (
    id INTEGER PRIMARY KEY,
    cart_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    FOREIGN KEY (cart_id) REFERENCES carts (id),
    FOREIGN KEY (product_id) REFERENCES products (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS orders (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id),
    fulfilled BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS order_items (
    id INTEGER PRIMARY KEY,
    order_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    FOREIGN KEY (order_id) REFERENCES orders (id),
    FOREIGN KEY (product_id) REFERENCES products (id) ON DELETE CASCADE
);
EOF

echo "Database tables recreated successfully."