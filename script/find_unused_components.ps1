# 查找未使用的 Vue 组件
# 此脚本会扫描所有 .vue 文件，并检查它们是否在项目中被引用

param(
    [string]$SrcPath = "C:\Users\liyue\Desktop\projects\dashboard\cutie\src",
    [switch]$Detailed = $false
)

Write-Host "=== 开始扫描未使用的组件 ===" -ForegroundColor Cyan
Write-Host ""

# 获取所有 Vue 组件文件
$allComponents = Get-ChildItem -Path $SrcPath -Filter "*.vue" -Recurse | Where-Object {
    # 排除 node_modules
    $_.FullName -notlike "*node_modules*"
}

Write-Host "总共找到 $($allComponents.Count) 个组件文件" -ForegroundColor Green
Write-Host ""

# 存储未使用的组件
$unusedComponents = @()
$usedComponents = @()

# 获取所有可能引用组件的文件（.vue, .ts, .js）
$allSourceFiles = Get-ChildItem -Path $SrcPath -Include "*.vue", "*.ts", "*.js" -Recurse | Where-Object {
    $_.FullName -notlike "*node_modules*"
}

$totalComponents = $allComponents.Count
$currentIndex = 0

foreach ($component in $allComponents) {
    $currentIndex++
    $componentName = $component.BaseName
    $relativePath = $component.FullName.Replace("$SrcPath\", "").Replace("\", "/")
    
    Write-Progress -Activity "检查组件" -Status "处理中: $componentName ($currentIndex/$totalComponents)" -PercentComplete (($currentIndex / $totalComponents) * 100)
    
    $isUsed = $false
    $usageCount = 0
    $usedIn = @()
    
    # 检查是否在其他文件中被引用
    foreach ($file in $allSourceFiles) {
        # 跳过组件自身
        if ($file.FullName -eq $component.FullName) {
            continue
        }
        
        $content = Get-Content $file.FullName -Raw -ErrorAction SilentlyContinue
        
        if ($content) {
            # 检查多种引用模式
            $patterns = @(
                # import 语句
                "import\s+.*\s+from\s+['""].*$componentName",
                "import\s+\{\s*.*$componentName.*\s*\}\s+from",
                # 动态 import
                "import\(['""].*$componentName",
                # 组件名称直接使用（在模板中）
                "<$componentName",
                # defineAsyncComponent
                "defineAsyncComponent.*['""].*$componentName"
            )
            
            foreach ($pattern in $patterns) {
                if ($content -match $pattern) {
                    $isUsed = $true
                    $usageCount++
                    $usedIn += $file.FullName.Replace("$SrcPath\", "").Replace("\", "/")
                    break
                }
            }
            
            if ($isUsed -and -not $Detailed) {
                break
            }
        }
    }
    
    if (-not $isUsed) {
        $unusedComponents += [PSCustomObject]@{
            Name     = $componentName
            Path     = $relativePath
            FullPath = $component.FullName
        }
    }
    else {
        $usedComponents += [PSCustomObject]@{
            Name       = $componentName
            Path       = $relativePath
            UsageCount = $usageCount
            UsedIn     = ($usedIn | Select-Object -Unique)
        }
    }
}

Write-Progress -Activity "检查组件" -Completed

Write-Host ""
Write-Host "=== 扫描结果 ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "已使用的组件: $($usedComponents.Count)" -ForegroundColor Green
Write-Host "未使用的组件: $($unusedComponents.Count)" -ForegroundColor Yellow
Write-Host ""

if ($unusedComponents.Count -gt 0) {
    Write-Host "=== 未使用的组件列表 ===" -ForegroundColor Red
    Write-Host ""
    
    $unusedComponents | Sort-Object Path | ForEach-Object {
        Write-Host "  ❌ $($_.Name)" -ForegroundColor Red
        Write-Host "     路径: $($_.Path)" -ForegroundColor Gray
        Write-Host ""
    }
    
    # 导出到文件
    $reportPath = "C:\Users\liyue\Desktop\projects\dashboard\cutie\unused_components_report.txt"
    
    $reportContent = @"
未使用的组件报告
生成时间: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
================================

总计: $($unusedComponents.Count) 个未使用的组件

"@
    
    foreach ($comp in ($unusedComponents | Sort-Object Path)) {
        $reportContent += "`n组件名: $($comp.Name)`n"
        $reportContent += "路径: $($comp.Path)`n"
        $reportContent += "完整路径: $($comp.FullPath)`n"
        $reportContent += "---`n"
    }
    
    $reportContent | Out-File -FilePath $reportPath -Encoding UTF8
    Write-Host "详细报告已保存到: $reportPath" -ForegroundColor Cyan
}

if ($Detailed -and $usedComponents.Count -gt 0) {
    Write-Host ""
    Write-Host "=== 已使用的组件详情 ===" -ForegroundColor Green
    Write-Host ""
    
    $usedComponents | Sort-Object Path | ForEach-Object {
        Write-Host "  ✓ $($_.Name)" -ForegroundColor Green
        Write-Host "     路径: $($_.Path)" -ForegroundColor Gray
        Write-Host "     引用次数: $($_.UsageCount)" -ForegroundColor Gray
        if ($_.UsedIn.Count -gt 0 -and $_.UsedIn.Count -le 5) {
            Write-Host "     使用位置:" -ForegroundColor Gray
            $_.UsedIn | ForEach-Object {
                Write-Host "       - $_" -ForegroundColor DarkGray
            }
        }
        Write-Host ""
    }
}

Write-Host ""
Write-Host "=== 扫描完成 ===" -ForegroundColor Cyan



