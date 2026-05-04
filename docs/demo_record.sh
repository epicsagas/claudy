#!/bin/bash
# Demo recording script for claudy
# Run: asciinema rec -c "USER=user HOST=localhost bash docs/demo_record.sh" docs/assets/demo.cast
set -e

cd "$(dirname "$0")/.."

echo ""
sleep 1

# --- Part 1: Overview ---
echo "$ claudy --version"
claudy --version
sleep 2

echo ""
echo "$ claudy ls"
claudy ls
sleep 3

echo ""
echo "$ claudy show zai"
claudy show zai
sleep 3

echo ""
echo "$ claudy ping zai"
claudy ping zai
sleep 3

echo ""
sleep 1

# --- Part 2: Launch Claude Code with Z.AI ---
claudy zai --yolo

sleep 2

# --- Part 3: Create epic mode and launch claude code with Z.AI ---

echo ""
echo "$ claudy mode ls"
claudy mode ls

sleep 2

echo ""
echo "$ claudy mode create epic"
claudy mode create epic

sleep 2

echo ""
echo "$ claudy zai epic --yolo"
claudy zai epic --yolo

sleep 2

clear

echo ""
echo "$ claudy analytics ingest"
claudy analytics ingest

echo ""
echo "$ claudy analytics recommend"
claudy analytics recommend

sleep 2
