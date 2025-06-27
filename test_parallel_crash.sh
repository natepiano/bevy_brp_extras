#!/bin/bash

# Test script to reproduce parallel execution crash with diagnostics

echo "=== Parallel Crash Test ==="
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

# Launch apps on different ports with debug output
(
    export BRP_PORT=17001
    export CARGO_MANIFEST_DIR="$(pwd)"
    echo "[17001] Starting extras_plugin..." > /tmp/parallel_test_17001.log
    cargo run --example extras_plugin 2>&1 | tee -a /tmp/parallel_test_17001.log &
    PID1=$!
    echo "[17001] PID: $PID1" >> /tmp/parallel_test_17001.log
    wait $PID1
    EXIT1=$?
    echo "[17001] Exit code: $EXIT1" >> /tmp/parallel_test_17001.log
    if [ $EXIT1 -eq 139 ]; then
        echo "[17001] SEGFAULT DETECTED!" >> /tmp/parallel_test_17001.log
    fi
) &

(
    export BRP_PORT=17002
    export CARGO_MANIFEST_DIR="$(pwd)"
    echo "[17002] Starting extras_plugin..." > /tmp/parallel_test_17002.log
    cargo run --example extras_plugin 2>&1 | tee -a /tmp/parallel_test_17002.log &
    PID2=$!
    echo "[17002] PID: $PID2" >> /tmp/parallel_test_17002.log
    wait $PID2
    EXIT2=$?
    echo "[17002] Exit code: $EXIT2" >> /tmp/parallel_test_17002.log
    if [ $EXIT2 -eq 139 ]; then
        echo "[17002] SEGFAULT DETECTED!" >> /tmp/parallel_test_17002.log
    fi
) &

(
    export BRP_PORT=17003
    export CARGO_MANIFEST_DIR="$(pwd)"
    echo "[17003] Starting extras_plugin..." > /tmp/parallel_test_17003.log
    cargo run --example extras_plugin 2>&1 | tee -a /tmp/parallel_test_17003.log &
    PID3=$!
    echo "[17003] PID: $PID3" >> /tmp/parallel_test_17003.log
    wait $PID3
    EXIT3=$?
    echo "[17003] Exit code: $EXIT3" >> /tmp/parallel_test_17003.log
    if [ $EXIT3 -eq 139 ]; then
        echo "[17003] SEGFAULT DETECTED!" >> /tmp/parallel_test_17003.log
    fi
) &

# Give apps time to start
sleep 5

echo "Running format discovery operations in parallel..."

# Run operations that might trigger crash
(
    sleep 1
    curl -X POST http://localhost:17001/bevy_brp_extras/discover_format \
        -H "Content-Type: application/json" \
        -d '{"types": ["bevy_transform::components::transform::Transform", "bevy_render::color::Color"]}' \
        > /tmp/parallel_test_17001_response.log 2>&1
) &

(
    sleep 1
    curl -X POST http://localhost:17002/bevy_brp_extras/discover_format \
        -H "Content-Type: application/json" \
        -d '{"types": ["bevy_transform::components::transform::Transform", "bevy_render::color::Color"]}' \
        > /tmp/parallel_test_17002_response.log 2>&1
) &

(
    sleep 1
    curl -X POST http://localhost:17003/bevy_brp_extras/discover_format \
        -H "Content-Type: application/json" \
        -d '{"types": ["bevy_transform::components::transform::Transform", "bevy_render::color::Color"]}' \
        > /tmp/parallel_test_17003_response.log 2>&1
) &

# Wait for operations
sleep 3

echo "Checking for crashes..."

# Check if apps are still running
for PORT in 17001 17002 17003; do
    if curl -s http://localhost:$PORT/rpc.discover > /dev/null 2>&1; then
        echo "App on port $PORT is still running"
    else
        echo "App on port $PORT has CRASHED or is not responding!"
        echo "Check /tmp/parallel_test_${PORT}.log for details"
    fi
done

# Look for core dumps
if ls core* 2>/dev/null; then
    echo "CORE DUMPS FOUND!"
    for core in core*; do
        echo "Analyzing $core..."
        lldb -c "$core" -b -o "bt all" -o "quit" 2>&1 | head -100
    done
fi

echo "Test complete. Check /tmp/parallel_test_*.log for details"