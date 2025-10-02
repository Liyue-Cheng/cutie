# 性能测试脚本 (PowerShell) - 测试 reopen 和 complete 任务的延迟

# 配置
$SIDECAR_URL = "http://localhost:8655"
$TASK_ID = "194a6b35-03e6-46f6-927f-d12335dda584"

Write-Host "=== Cutie Task Performance Test ===" -ForegroundColor Cyan
Write-Host "Sidecar URL: $SIDECAR_URL"
Write-Host "Task ID: $TASK_ID"
Write-Host ""

# 测试函数
function Test-Endpoint {
    param(
        [string]$Method,
        [string]$Url,
        [string]$Name,
        [int]$Iterations = 10
    )
    
    Write-Host "=== Testing $Name ($Iterations iterations) ===" -ForegroundColor Yellow
    $times = @()
    
    for ($i = 1; $i -le $Iterations; $i++) {
        $correlationId = "test-$Name-$i"
        $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
        
        try {
            $headers = @{
                "X-Correlation-ID" = $correlationId
            }
            
            if ($Method -eq "DELETE") {
                $response = Invoke-WebRequest -Uri $Url -Method Delete -Headers $headers -UseBasicParsing -ErrorAction Stop
            }
            else {
                $response = Invoke-WebRequest -Uri $Url -Method Post -Headers $headers -UseBasicParsing -ErrorAction Stop
            }
            
            $stopwatch.Stop()
            $elapsed = $stopwatch.Elapsed.TotalMilliseconds
            $times += $elapsed
            
            Write-Host ("Run {0,2}: {1,7:N2} ms | Status: {2}" -f $i, $elapsed, $response.StatusCode)
        }
        catch {
            $stopwatch.Stop()
            $elapsed = $stopwatch.Elapsed.TotalMilliseconds
            Write-Host ("Run {0,2}: {1,7:N2} ms | ERROR: {2}" -f $i, $elapsed, $_.Exception.Message) -ForegroundColor Red
        }
        
        Start-Sleep -Milliseconds 100
    }
    
    if ($times.Count -gt 0) {
        $avg = ($times | Measure-Object -Average).Average
        $min = ($times | Measure-Object -Minimum).Minimum
        $max = ($times | Measure-Object -Maximum).Maximum
        
        Write-Host ("-" * 50) -ForegroundColor Gray
        Write-Host ("Summary: Avg={0:N2}ms | Min={1:N2}ms | Max={2:N2}ms" -f $avg, $min, $max) -ForegroundColor Green
    }
    
    Write-Host ""
}

# 测试 Complete Task
Test-Endpoint -Method "POST" -Url "$SIDECAR_URL/api/tasks/$TASK_ID/completion" -Name "COMPLETE" -Iterations 10

# 测试 Reopen Task
Test-Endpoint -Method "DELETE" -Url "$SIDECAR_URL/api/tasks/$TASK_ID/completion" -Name "REOPEN" -Iterations 10

Write-Host "=== Test Complete ===" -ForegroundColor Cyan

