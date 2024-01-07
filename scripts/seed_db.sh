#!/bin/bash

DATABASE_URL="database.db"

# Seed database
sqlite3 $DATABASE_URL <<EOF
INSERT OR IGNORE INTO products (name, image, price, description) VALUES
    ('Runtz', 'https://images.leafly.com/flower-images/runtz-nug-image.jpg', 1000, '')
EOF

echo "Database seeded successfully."