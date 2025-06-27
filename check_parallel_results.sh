#!/bin/bash

echo "=== Checking Parallel Test Results ==="

# Check if apps are still running
for PORT in 17001 17002 17003; do
    echo -n "Port $PORT: "
    if curl -s http://localhost:$PORT/rpc.discover > /dev/null 2>&1; then
        echo "App RUNNING"
    else
        echo "App CRASHED or NOT RESPONDING"
        
        # Check exit code from log
        if [ -f /tmp/parallel_test_${PORT}.log ]; then
            EXIT_CODE=$(grep "Exit code:" /tmp/parallel_test_${PORT}.log | tail -1)
            if [ ! -z "$EXIT_CODE" ]; then
                echo "  $EXIT_CODE"
            fi
            
            # Check for segfault
            if grep -q "SEGFAULT" /tmp/parallel_test_${PORT}.log; then
                echo "  *** SEGMENTATION FAULT DETECTED ***"
            fi
        fi
    fi
done

echo ""
echo "=== HTTP Response Results ==="
for PORT in 17001 17002 17003; do
    if [ -f /tmp/parallel_test_${PORT}_response.log ]; then
        echo "Port $PORT response:"
        head -1 /tmp/parallel_test_${PORT}_response.log
    fi
done

echo ""
echo "=== Checking for Core Dumps ==="
if ls core* 2>/dev/null; then
    echo "CORE DUMPS FOUND!"
    ls -la core*
else
    echo "No core dumps found"
fi

echo ""
echo "=== Process Status ==="
ps aux | grep -E "(extras_plugin|cargo run)" | grep -v grep || echo "No Bevy processes found"

echo ""
echo "For detailed logs, check:"
echo "  tail -f /tmp/parallel_test_*.log"