#!/bin/bash

# Test script to reproduce parallel execution crash with diagnostics
# This version runs in background with nohup

echo "=== Parallel Crash Test (Background) ==="
echo "Enabling crash diagnostics..."

# Enable core dumps
ulimit -c unlimited

# Clean up old logs
rm -f /tmp/bevy_brp_mcp_*.log
rm -f /tmp/parallel_test_*.log

# Set environment for better debugging
export RUST_BACKTRACE=full
export RUST_LIB_BACKTRACE=full

echo "Starting 3 apps in parallel with debug logging..."

# Launch apps on different ports with nohup
nohup bash -c '
    export BRP_PORT=17001
    export CARGO_MANIFEST_DIR="$(pwd)"
    echo "[17001] Starting extras_plugin..." > /tmp/parallel_test_17001.log
    cargo run --example extras_plugin >> /tmp/parallel_test_17001.log 2>&1 &
    PID=$!
    echo "[17001] PID: $PID" >> /tmp/parallel_test_17001.log
    wait $PID
    EXIT=$?
    echo "[17001] Exit code: $EXIT" >> /tmp/parallel_test_17001.log
    if [ $EXIT -eq 139 ]; then
        echo "[17001] SEGFAULT DETECTED!" >> /tmp/parallel_test_17001.log
    fi
' > /tmp/parallel_test_17001_nohup.out 2>&1 &

nohup bash -c '
    export BRP_PORT=17002
    export CARGO_MANIFEST_DIR="$(pwd)"
    echo "[17002] Starting extras_plugin..." > /tmp/parallel_test_17002.log
    cargo run --example extras_plugin >> /tmp/parallel_test_17002.log 2>&1 &
    PID=$!
    echo "[17002] PID: $PID" >> /tmp/parallel_test_17002.log
    wait $PID
    EXIT=$?
    echo "[17002] Exit code: $EXIT" >> /tmp/parallel_test_17002.log
    if [ $EXIT -eq 139 ]; then
        echo "[17002] SEGFAULT DETECTED!" >> /tmp/parallel_test_17002.log
    fi
' > /tmp/parallel_test_17002_nohup.out 2>&1 &

nohup bash -c '
    export BRP_PORT=17003
    export CARGO_MANIFEST_DIR="$(pwd)"
    echo "[17003] Starting extras_plugin..." > /tmp/parallel_test_17003.log
    cargo run --example extras_plugin >> /tmp/parallel_test_17003.log 2>&1 &
    PID=$!
    echo "[17003] PID: $PID" >> /tmp/parallel_test_17003.log
    wait $PID
    EXIT=$?
    echo "[17003] Exit code: $EXIT" >> /tmp/parallel_test_17003.log
    if [ $EXIT -eq 139 ]; then
        echo "[17003] SEGFAULT DETECTED!" >> /tmp/parallel_test_17003.log
    fi
' > /tmp/parallel_test_17003_nohup.out 2>&1 &

echo "Apps launching in background..."
echo "Waiting 10 seconds for apps to start..."
sleep 10

echo "Running format discovery operations in parallel..."

# Run operations in background
nohup curl -X POST http://localhost:17001/bevy_brp_extras/discover_format \
    -H "Content-Type: application/json" \
    -d '{"types": ["bevy_transform::components::transform::Transform", "bevy_render::color::Color"]}' \
    > /tmp/parallel_test_17001_response.log 2>&1 &

nohup curl -X POST http://localhost:17002/bevy_brp_extras/discover_format \
    -H "Content-Type: application/json" \
    -d '{"types": ["bevy_transform::components::transform::Transform", "bevy_render::color::Color"]}' \
    > /tmp/parallel_test_17002_response.log 2>&1 &

nohup curl -X POST http://localhost:17003/bevy_brp_extras/discover_format \
    -H "Content-Type: application/json" \
    -d '{"types": ["bevy_transform::components::transform::Transform", "bevy_render::color::Color"]}' \
    > /tmp/parallel_test_17003_response.log 2>&1 &

echo "Test operations launched. Check results with:"
echo "  ./check_parallel_results.sh"
echo ""
echo "Log files:"
echo "  /tmp/parallel_test_*.log"
echo "  /tmp/parallel_test_*_response.log"