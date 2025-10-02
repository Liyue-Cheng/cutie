# 性能测试脚本 (PowerShell) - 测试 reopen 和 complete 任务的延迟 + SSE 丢包检测

# 配置
$SIDECAR_URL = "http://localhost:10494"
$TASK_ID = "194a6b35-03e6-46f6-927f-d12335dda584"
$SSE_TIMEOUT_MS = 1000  # SSE 事件超时阈值（毫秒）

Write-Host "=== Cutie Task Performance Test with SSE Monitoring ===" -ForegroundColor Cyan
Write-Host "Sidecar URL: $SIDECAR_URL"
Write-Host "Task ID: $TASK_ID"
Write-Host "SSE Timeout: $SSE_TIMEOUT_MS ms"
Write-Host ""

# 全局变量：追踪待确认的 correlation IDs
$script:PendingCorrelations = [System.Collections.Concurrent.ConcurrentDictionary[string, object]]::new()
$script:ReceivedEvents = [System.Collections.Concurrent.ConcurrentBag[object]]::new()
$script:SSEConnected = $false
$script:SSEErrors = 0
$script:PacketLoss = 0

# SSE 监听后台任务
function Start-SSEMonitor {
    $script:SSEJob = Start-Job -ScriptBlock {
        param($Url, $TimeoutMs)
        
        try {
            $client = New-Object System.Net.Http.HttpClient
            $client.Timeout = [TimeSpan]::FromMinutes(10)
            
            $request = New-Object System.Net.Http.HttpRequestMessage([System.Net.Http.HttpMethod]::Get, "$Url/api/events/stream")
            $request.Headers.Accept.Add([System.Net.Http.Headers.MediaTypeWithQualityHeaderValue]::new("text/event-stream"))
            
            Write-Output "[SSE] Connecting to event stream..."
            $response = $client.SendAsync($request, [System.Net.Http.HttpCompletionOption]::ResponseHeadersRead).Result
            
            if ($response.IsSuccessStatusCode) {
                Write-Output "[SSE] Connected successfully"
                $stream = $response.Content.ReadAsStreamAsync().Result
                $reader = New-Object System.IO.StreamReader($stream)
                
                $eventType = $null
                $eventId = $null
                $eventData = $null
                
                while (-not $reader.EndOfStream) {
                    $line = $reader.ReadLine()
                    
                    if ($line -match '^event:\s*(.+)$') {
                        $eventType = $matches[1]
                    }
                    elseif ($line -match '^id:\s*(.+)$') {
                        $eventId = $matches[1]
                    }
                    elseif ($line -match '^data:\s*(.+)$') {
                        $eventData = $matches[1]
                    }
                    elseif ($line -eq '' -and $eventType -and $eventData) {
                        # 完整事件接收
                        try {
                            $event = $eventData | ConvertFrom-Json
                            $timestamp = Get-Date -Format "HH:mm:ss.fff"
                            
                            Write-Output "[SSE] [$timestamp] Event: $eventType | correlation_id: $($event.correlation_id)"
                            
                            # 输出事件信息供主进程处理
                            @{
                                Type          = $eventType
                                CorrelationId = $event.correlation_id
                                EventId       = $eventId
                                Timestamp     = $timestamp
                                Data          = $event
                            } | ConvertTo-Json -Compress
                        }
                        catch {
                            Write-Output "[SSE] Error parsing event: $_"
                        }
                        
                        # 重置
                        $eventType = $null
                        $eventId = $null
                        $eventData = $null
                    }
                }
            }
            else {
                Write-Output "[SSE] Connection failed: $($response.StatusCode)"
            }
        }
        catch {
            Write-Output "[SSE] Error: $_"
        }
        finally {
            if ($client) { $client.Dispose() }
        }
    } -ArgumentList $SIDECAR_URL, $SSE_TIMEOUT_MS
    
    # 等待连接建立
    Start-Sleep -Milliseconds 500
    Write-Host "[SSE Monitor] Started (Job ID: $($script:SSEJob.Id))" -ForegroundColor Cyan
}

# 处理 SSE 事件
function Process-SSEEvents {
    if ($script:SSEJob) {
        $output = Receive-Job -Job $script:SSEJob -Keep
        foreach ($line in $output) {
            if ($line -match '^\{.*\}$') {
                try {
                    $event = $line | ConvertFrom-Json
                    
                    if ($event.CorrelationId) {
                        # 标记为已接收
                        if ($script:PendingCorrelations.ContainsKey($event.CorrelationId)) {
                            $pending = $script:PendingCorrelations[$event.CorrelationId]
                            $sseDelay = ((Get-Date) - $pending.SentAt).TotalMilliseconds
                            
                            Write-Host ("    ✓ SSE Event received | Δ={0:N2}ms | correlation: {1}" -f $sseDelay, $event.CorrelationId) -ForegroundColor Green
                            
                            [void]$script:PendingCorrelations.TryRemove($event.CorrelationId, [ref]$null)
                            $script:ReceivedEvents.Add($event)
                        }
                    }
                }
                catch {
                    # 非 JSON 行，忽略
                }
            }
        }
    }
}

# 检查 SSE 超时
function Check-SSETimeouts {
    $now = Get-Date
    $timedOut = @()
    
    foreach ($kvp in $script:PendingCorrelations.GetEnumerator()) {
        $elapsed = ($now - $kvp.Value.SentAt).TotalMilliseconds
        if ($elapsed -gt $SSE_TIMEOUT_MS) {
            $timedOut += $kvp.Key
        }
    }
    
    foreach ($correlationId in $timedOut) {
        $pending = $null
        if ($script:PendingCorrelations.TryRemove($correlationId, [ref]$pending)) {
            $elapsed = ($now - $pending.SentAt).TotalMilliseconds
            Write-Host ("    ⚠ SSE PACKET LOSS! No event after {0:N0}ms | correlation: {1}" -f $elapsed, $correlationId) -ForegroundColor Red
            $script:PacketLoss++
        }
    }
}

# 测试函数（增强版，带 SSE 跟踪）
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
        $correlationId = "test-$Name-$i-$(Get-Date -Format 'HHmmss')"
        $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
        
        try {
            $headers = @{
                "X-Correlation-ID" = $correlationId
            }
            
            # 记录发送时间
            $pendingEntry = @{
                Name      = $Name
                SentAt    = Get-Date
                Iteration = $i
            }
            $script:PendingCorrelations[$correlationId] = $pendingEntry
            
            if ($Method -eq "DELETE") {
                $response = Invoke-WebRequest -Uri $Url -Method Delete -Headers $headers -UseBasicParsing -ErrorAction Stop
            }
            else {
                $response = Invoke-WebRequest -Uri $Url -Method Post -Headers $headers -UseBasicParsing -ErrorAction Stop
            }
            
            $stopwatch.Stop()
            $elapsed = $stopwatch.Elapsed.TotalMilliseconds
            $times += $elapsed
            
            Write-Host ("Run {0,2}: {1,7:N2} ms | HTTP Status: {2} | correlation: {3}" -f $i, $elapsed, $response.StatusCode, $correlationId)
        }
        catch {
            $stopwatch.Stop()
            $elapsed = $stopwatch.Elapsed.TotalMilliseconds
            Write-Host ("Run {0,2}: {1,7:N2} ms | ERROR: {2}" -f $i, $elapsed, $_.Exception.Message) -ForegroundColor Red
            [void]$script:PendingCorrelations.TryRemove($correlationId, [ref]$null)
        }
        
        # 快速处理 SSE 事件（不阻塞）
        Process-SSEEvents
    }
    
    if ($times.Count -gt 0) {
        $avg = ($times | Measure-Object -Average).Average
        $min = ($times | Measure-Object -Minimum).Minimum
        $max = ($times | Measure-Object -Maximum).Maximum
        
        Write-Host ("-" * 60) -ForegroundColor Gray
        Write-Host ("HTTP Summary: Avg={0:N2}ms | Min={1:N2}ms | Max={2:N2}ms" -f $avg, $min, $max) -ForegroundColor Green
    }
    
    Write-Host ""
}

# 启动 SSE 监控
Start-SSEMonitor

# 等待 SSE 连接稳定
Write-Host "Waiting for SSE connection to stabilize..." -ForegroundColor Yellow
Start-Sleep -Seconds 2

# 轮替测试 Complete 和 Reopen（连续发请求，不等待 SSE）
Write-Host "=== Testing COMPLETE/REOPEN alternating (20 iterations) ===" -ForegroundColor Yellow
$times = @()
$Iterations = 20

for ($i = 1; $i -le $Iterations; $i++) {
    $isComplete = ($i % 2 -eq 1)  # 奇数次 complete，偶数次 reopen
    $method = if ($isComplete) { "POST" } else { "DELETE" }
    $name = if ($isComplete) { "COMPLETE" } else { "REOPEN" }
    
    $correlationId = "test-$name-$i-$(Get-Date -Format 'HHmmssffff')"
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    
    try {
        $headers = @{
            "X-Correlation-ID" = $correlationId
        }
        
        # 记录发送时间
        $pendingEntry = @{
            Name      = $name
            SentAt    = Get-Date
            Iteration = $i
        }
        $script:PendingCorrelations[$correlationId] = $pendingEntry
        
        if ($method -eq "DELETE") {
            $response = Invoke-WebRequest -Uri "$SIDECAR_URL/api/tasks/$TASK_ID/completion" -Method Delete -Headers $headers -UseBasicParsing -ErrorAction Stop
        }
        else {
            $response = Invoke-WebRequest -Uri "$SIDECAR_URL/api/tasks/$TASK_ID/completion" -Method Post -Headers $headers -UseBasicParsing -ErrorAction Stop
        }
        
        $stopwatch.Stop()
        $elapsed = $stopwatch.Elapsed.TotalMilliseconds
        $times += $elapsed
        
        Write-Host ("Run {0,2} ({1,8}): {2,7:N2} ms | HTTP Status: {3} | correlation: {4}" -f $i, $name, $elapsed, $response.StatusCode, $correlationId)
    }
    catch {
        $stopwatch.Stop()
        $elapsed = $stopwatch.Elapsed.TotalMilliseconds
        Write-Host ("Run {0,2} ({1,8}): {2,7:N2} ms | ERROR: {3}" -f $i, $name, $elapsed, $_.Exception.Message) -ForegroundColor Red
        [void]$script:PendingCorrelations.TryRemove($correlationId, [ref]$null)
    }
    
    # 快速处理 SSE 事件（不阻塞）
    Process-SSEEvents
}

if ($times.Count -gt 0) {
    $avg = ($times | Measure-Object -Average).Average
    $min = ($times | Measure-Object -Minimum).Minimum
    $max = ($times | Measure-Object -Maximum).Maximum
    
    Write-Host ("-" * 60) -ForegroundColor Gray
    Write-Host ("HTTP Summary: Avg={0:N2}ms | Min={1:N2}ms | Max={2:N2}ms" -f $avg, $min, $max) -ForegroundColor Green
}

Write-Host ""

# 等待所有 SSE 事件（最多等待 5 秒）
Write-Host "Waiting for remaining SSE events (max 5s)..." -ForegroundColor Yellow
for ($i = 0; $i -lt 25; $i++) {
    Process-SSEEvents
    Check-SSETimeouts
    
    if ($script:PendingCorrelations.Count -eq 0) {
        Write-Host "All SSE events received!" -ForegroundColor Green
        break
    }
    
    Start-Sleep -Milliseconds 200
}

# 最终报告
Write-Host ""
Write-Host "=== SSE Event Monitoring Summary ===" -ForegroundColor Cyan
Write-Host ("Total SSE Events Received: {0}" -f $script:ReceivedEvents.Count) -ForegroundColor Green
Write-Host ("Pending (not received): {0}" -f $script:PendingCorrelations.Count) -ForegroundColor Yellow
Write-Host ("Packet Loss Detected: {0}" -f $script:PacketLoss) -ForegroundColor $(if ($script:PacketLoss -eq 0) { "Green" } else { "Red" })

if ($script:PendingCorrelations.Count -gt 0) {
    Write-Host "`nStill waiting for:" -ForegroundColor Yellow
    foreach ($kvp in $script:PendingCorrelations.GetEnumerator()) {
        Write-Host ("  - {0}" -f $kvp.Key) -ForegroundColor Yellow
    }
}

# 清理
if ($script:SSEJob) {
    Stop-Job -Job $script:SSEJob
    Remove-Job -Job $script:SSEJob
    Write-Host "`n[SSE Monitor] Stopped" -ForegroundColor Cyan
}

Write-Host "`n=== Test Complete ===" -ForegroundColor Cyan
