#!/usr/bin/env bash

echo "Rust version:"
rustc --version

echo "### Updating package list..."
sudo apt-get update
sudo apt-get upgrade -y

echo "### Installing PostgreSQL client..."
sudo apt-get install -y postgresql-client

echo "### Installing sea-orm-cli..."
cargo install sea-orm-cli@^2.0.0-rc

echo "### Setup complete!"
