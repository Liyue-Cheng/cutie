#!/bin/bash
# 性能测试脚本 - 测试 reopen 和 complete 任务的延迟

# 配置
SIDECAR_URL="http://localhost:3721"
TASK_ID="194a6b35-03e6-46f6-927f-d12335dda584"

echo "=== Cutie Task Performance Test ==="
echo "Sidecar URL: $SIDECAR_URL"
echo "Task ID: $TASK_ID"
echo ""

# 测试 1: Complete Task (10次)
echo "=== Testing COMPLETE TASK (10 iterations) ==="
for i in {1..10}; do
  echo -n "Run $i: "
  curl -w "@curl-format.txt" -o /dev/null -s \
    -X POST \
    -H "X-Correlation-ID: test-complete-$i" \
    "$SIDECAR_URL/api/tasks/$TASK_ID/completion"
  echo ""
done

echo ""
echo "=== Testing REOPEN TASK (10 iterations) ==="
for i in {1..10}; do
  echo -n "Run $i: "
  curl -w "@curl-format.txt" -o /dev/null -s \
    -X DELETE \
    -H "X-Correlation-ID: test-reopen-$i" \
    "$SIDECAR_URL/api/tasks/$TASK_ID/completion"
  echo ""
done

echo ""
echo "=== Test Complete ==="

