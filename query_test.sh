#!/bin/bash
EVENT_ID=$1
if [ -z "$EVENT_ID" ]; then
  echo "Usage: ./query_test.sh <EVENT_ID>"
  exit 1
fi
podman exec cli bash -c "peer chaincode query -C dwntpchannel -n dwntp -c '{\"function\":\"QueryEvent\",\"Args\":[\"$EVENT_ID\"]}'"
