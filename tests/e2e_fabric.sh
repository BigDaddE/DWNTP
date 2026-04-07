#!/bin/bash
set -e

echo "==========================================="
echo "   DWNTP Hyperledger Fabric E2E Test"
echo "==========================================="

# Ensure we are in the project root
cd "$(dirname "$0")/.."

echo "Building dwntp-client..."
cargo build --bin dwntp-client

CLIENT="cargo run -q --bin dwntp-client --"

echo -e "\n[Test 1] Logging first event..."
RTU_ID_1="RTU-E2E-$(date +%s)-1"
LOG_OUTPUT_1=$($CLIENT log-event --source-mtu "E2E-MTU-01" --rtu-id "$RTU_ID_1" --event-name "BreakerOpen" --event-desc "Open main breaker for E2E test")

echo "$LOG_OUTPUT_1"

# Extract the event ID from the output
EVENT_ID_1=$(echo "$LOG_OUTPUT_1" | grep "Event ID:" | awk '{print $3}')

if [ -z "$EVENT_ID_1" ]; then
    echo "❌ ERROR: Failed to extract Event ID from LogEvent output."
    exit 1
fi
echo "✅ Successfully logged event. Event ID: $EVENT_ID_1"

echo "Waiting for transaction to be committed..."
sleep 3

echo -e "\n[Test 2] Querying the specific event by ID..."
QUERY_OUTPUT_1=$($CLIENT query-event --id "$EVENT_ID_1")
echo "$QUERY_OUTPUT_1"

if echo "$QUERY_OUTPUT_1" | grep -q "$RTU_ID_1"; then
    echo "✅ Successfully queried event and matched RTU ID ($RTU_ID_1)."
else
    echo "❌ ERROR: Queried event did not contain expected RTU ID."
    exit 1
fi


echo -e "\n[Test 3] Logging a second event..."
sleep 1 # Ensure different timestamp
RTU_ID_2="RTU-E2E-$(date +%s)-2"
LOG_OUTPUT_2=$($CLIENT log-event --source-mtu "E2E-MTU-02" --rtu-id "$RTU_ID_2" --event-name "SetVoltage" --event-desc "Set voltage to 240V for E2E test")

EVENT_ID_2=$(echo "$LOG_OUTPUT_2" | grep "Event ID:" | awk '{print $3}')
if [ -z "$EVENT_ID_2" ]; then
    echo "❌ ERROR: Failed to extract second Event ID."
    exit 1
fi
echo "✅ Successfully logged second event. Event ID: $EVENT_ID_2"

echo "Waiting for transaction to be committed..."
sleep 3

echo -e "\n[Test 4] Retrieving all events from the ledger..."
ALL_EVENTS_OUTPUT=$($CLIENT get-all-events)

# Check if both events are present in the get-all-events output
if echo "$ALL_EVENTS_OUTPUT" | grep -q "$EVENT_ID_1" && echo "$ALL_EVENTS_OUTPUT" | grep -q "$EVENT_ID_2"; then
    echo "✅ Successfully retrieved all events. Both E2E events were found in the ledger."
else
    echo "❌ ERROR: 'get-all-events' output is missing one or both of our test events."
    echo "Output:"
    echo "$ALL_EVENTS_OUTPUT"
    exit 1
fi

echo -e "\n==========================================="
echo " 🎉 ALL END-TO-END TESTS PASSED SUCCESSFULLY! "
echo "==========================================="
