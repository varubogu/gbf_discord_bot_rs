#!/usr/bin/env bash

echo "Rust version:"
rustc --version

echo "### Updating package list..."
sudo apt-get update
sudo apt-get upgrade -y

echo "### Installing PostgreSQL client..."
sudo apt-get install -y postgresql-client

echo "### Setup complete!"