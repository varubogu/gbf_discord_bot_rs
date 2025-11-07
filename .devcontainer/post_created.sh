#!/usr/bin/env bash

echo "Rust version:"
rustc --version

echo "### Updating package list..."
sudo apt-get update

echo "### Installing PostgreSQL client..."
sudo apt-get install -y postgresql-client

echo "### Setup complete!"