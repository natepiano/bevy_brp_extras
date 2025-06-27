#!/bin/bash

# Debug launcher for Bevy apps with crash detection
APP_NAME="$1"
PORT="${2:-15702}"
LOG_FILE="/tmp/bevy_brp_mcp_${APP_NAME}_$(date +%s).log"
CRASH_LOG="/tmp/bevy_brp_mcp_${APP_NAME}_crash_$(date +%s).log"

echo "=== Debug Launcher Starting ===" > "$CRASH_LOG"
echo "App: $APP_NAME" >> "$CRASH_LOG"
echo "Port: $PORT" >> "$CRASH_LOG"
echo "Time: $(date)" >> "$CRASH_LOG"
echo "===========================" >> "$CRASH_LOG"

# Enable core dumps
ulimit -c unlimited

# Set up signal handlers
trap 'echo "Process received SIGINT" >> "$CRASH_LOG"' INT
trap 'echo "Process received SIGTERM" >> "$CRASH_LOG"' TERM

# Run with environment variables and capture both stdout/stderr
export RUST_BACKTRACE=full
export BRP_PORT="$PORT"
export CARGO_MANIFEST_DIR="$(pwd)"

echo "Running: cargo run --example $APP_NAME" >> "$CRASH_LOG"

# Run the app and capture exit code
cargo run --example "$APP_NAME" >> "$LOG_FILE" 2>&1 &
PID=$!

echo "Started with PID: $PID" >> "$CRASH_LOG"

# Wait for the process and capture exit code
wait $PID
EXIT_CODE=$?

echo "Process exited with code: $EXIT_CODE" >> "$CRASH_LOG"

# Check for segfault or other signals
if [ $EXIT_CODE -eq 139 ]; then
    echo "SEGMENTATION FAULT DETECTED!" >> "$CRASH_LOG"
elif [ $EXIT_CODE -eq 134 ]; then
    echo "ABORT SIGNAL DETECTED!" >> "$CRASH_LOG"
elif [ $EXIT_CODE -ne 0 ]; then
    echo "Abnormal exit detected" >> "$CRASH_LOG"
fi

# Look for core dump
if [ -f core ]; then
    echo "Core dump found!" >> "$CRASH_LOG"
    lldb -c core -b -o "bt all" -o "quit" >> "$CRASH_LOG" 2>&1
    mv core "core.${APP_NAME}.$(date +%s)"
fi

# Extract last 50 lines of app log
echo "=== Last 50 lines of app log ===" >> "$CRASH_LOG"
tail -50 "$LOG_FILE" >> "$CRASH_LOG"

echo "=== Debug launcher finished ===" >> "$CRASH_LOG"
echo "Exit code: $EXIT_CODE" >> "$CRASH_LOG"

# Print crash log location if abnormal exit
if [ $EXIT_CODE -ne 0 ]; then
    echo "CRASH_LOG:$CRASH_LOG"
fi