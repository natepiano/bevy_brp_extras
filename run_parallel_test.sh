#!/bin/bash

echo "=== Running Parallel Test with Pre-built Binary ==="

# Clean up
rm -f /tmp/parallel_crash_*.log
pkill -f "extras_plugin"
sleep 1

# Launch 3 instances directly
echo "Launching 3 instances..."
BRP_PORT=17001 RUST_BACKTRACE=1 ./target/debug/examples/extras_plugin > /tmp/parallel_crash_17001.log 2>&1 &
PID1=$!
echo "Started on port 17001, PID: $PID1"

BRP_PORT=17002 RUST_BACKTRACE=1 ./target/debug/examples/extras_plugin > /tmp/parallel_crash_17002.log 2>&1 &
PID2=$!
echo "Started on port 17002, PID: $PID2"

BRP_PORT=17003 RUST_BACKTRACE=1 ./target/debug/examples/extras_plugin > /tmp/parallel_crash_17003.log 2>&1 &
PID3=$!
echo "Started on port 17003, PID: $PID3"

echo "Waiting for apps to initialize..."
sleep 3

echo "Testing BRP connectivity..."
for PORT in 17001 17002 17003; do
    if curl -s http://localhost:$PORT/rpc.discover > /dev/null 2>&1; then
        echo "Port $PORT: Connected"
    else
        echo "Port $PORT: FAILED to connect"
    fi
done

echo ""
echo "Running simultaneous format discovery..."
curl -X POST http://localhost:17001/bevy_brp_extras/discover_format \
    -H "Content-Type: application/json" \
    -d '{"types": ["bevy_transform::components::transform::Transform", "bevy_render::color::Color"]}' \
    > /tmp/parallel_crash_17001_resp.log 2>&1 &

curl -X POST http://localhost:17002/bevy_brp_extras/discover_format \
    -H "Content-Type: application/json" \
    -d '{"types": ["bevy_transform::components::transform::Transform", "bevy_render::color::Color"]}' \
    > /tmp/parallel_crash_17002_resp.log 2>&1 &

curl -X POST http://localhost:17003/bevy_brp_extras/discover_format \
    -H "Content-Type: application/json" \
    -d '{"types": ["bevy_transform::components::transform::Transform", "bevy_render::color::Color"]}' \
    > /tmp/parallel_crash_17003_resp.log 2>&1 &

echo "Waiting for operations to complete..."
sleep 2

echo ""
echo "Checking app status after operations..."
for PORT in 17001 17002 17003; do
    if ps -p ${!PID} > /dev/null 2>&1; then
        if curl -s http://localhost:$PORT/rpc.discover > /dev/null 2>&1; then
            echo "Port $PORT: Still running and responsive"
        else
            echo "Port $PORT: Process alive but NOT RESPONDING"
        fi
    else
        echo "Port $PORT: CRASHED"
        echo "Check /tmp/parallel_crash_${PORT}.log for details"
    fi
done

echo ""
echo "Response summaries:"
for PORT in 17001 17002 17003; do
    if [ -f /tmp/parallel_crash_${PORT}_resp.log ]; then
        echo -n "Port $PORT: "
        if grep -q "success" /tmp/parallel_crash_${PORT}_resp.log; then
            echo "Success"
        else
            echo "Failed - $(head -1 /tmp/parallel_crash_${PORT}_resp.log)"
        fi
    fi
done

echo ""
echo "To cleanup: pkill -f extras_plugin"