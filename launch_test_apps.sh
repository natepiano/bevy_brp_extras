#!/bin/bash

echo "=== Launching Test Apps ==="
echo "Building first..."
cargo build --example extras_plugin

echo ""
echo "Launching apps for tests on configured ports..."

# Launch app for test 1 (port 20101)
echo -n "Launching on port 20101... "
nohup bash -c 'BRP_PORT=20101 RUST_BACKTRACE=1 ./target/debug/examples/extras_plugin' > /tmp/test_app_20101.log 2>&1 &
echo "PID: $!"

# Launch app for test 2 (port 20102)
echo -n "Launching on port 20102... "
nohup bash -c 'BRP_PORT=20102 RUST_BACKTRACE=1 ./target/debug/examples/extras_plugin' > /tmp/test_app_20102.log 2>&1 &
echo "PID: $!"

# Launch app for test 3 (port 20103)
echo -n "Launching on port 20103... "
nohup bash -c 'BRP_PORT=20103 RUST_BACKTRACE=1 ./target/debug/examples/extras_plugin' > /tmp/test_app_20103.log 2>&1 &
echo "PID: $!"

# Launch app for test 4 (port 20104)
echo -n "Launching on port 20104... "
nohup bash -c 'BRP_PORT=20104 RUST_BACKTRACE=1 ./target/debug/examples/extras_plugin' > /tmp/test_app_20104.log 2>&1 &
echo "PID: $!"

# Test 5 doesn't need an app

# Launch app for test 6 (port 20105)
echo -n "Launching on port 20105... "
nohup bash -c 'BRP_PORT=20105 RUST_BACKTRACE=1 ./target/debug/examples/extras_plugin' > /tmp/test_app_20105.log 2>&1 &
echo "PID: $!"

# Launch app for test 7 (port 20106)
echo -n "Launching on port 20106... "
nohup bash -c 'BRP_PORT=20106 RUST_BACKTRACE=1 ./target/debug/examples/extras_plugin' > /tmp/test_app_20106.log 2>&1 &
echo "PID: $!"

echo ""
echo "Waiting for apps to initialize..."
sleep 3

echo ""
echo "Checking app status:"
for PORT in 20101 20102 20103 20104 20105 20106; do
    echo -n "Port $PORT: "
    if curl -s -X POST http://localhost:$PORT/ \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"rpc.discover","id":1}' > /dev/null 2>&1; then
        echo "Ready"
    else
        echo "Not responding (may still be starting)"
    fi
done

echo ""
echo "Apps launched. Logs available at /tmp/test_app_*.log"
echo "To shutdown all: pkill -f extras_plugin"