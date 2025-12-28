#!/bin/bash
i=1
while [ $i -le 20 ]; do
  echo "=== Check $i ==="
  curl -s http://127.0.0.1:54321/api/entities | jq -r '.entities[] | "Pos:(\(.position.x),\(.position.y)) | \(.current_action) | H:\(.hunger)% T:\(.thirst)% E:\(.energy)%"'
  sleep 2
  i=$((i+1))
done
