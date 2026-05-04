#!/bin/bash
# Demo recording script for claudy
# Run: asciinema rec -c "bash docs/demo_record.sh" docs/assets/demo.cast
set -e

cd /Users/hackme/projects/claudy

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
exec claudy zai --yolo
