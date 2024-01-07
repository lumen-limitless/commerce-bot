#!/bin/bash

DATABASE_URL="database.db"

# Drop tables if they exist
sqlite3 $DATABASE_URL <<EOF
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS carts;
DROP TABLE IF EXISTS cart_items;
DROP TABLE IF EXISTS orders;
DROP TABLE IF EXISTS order_items;
EOF

echo "Database tables dropped successfully."