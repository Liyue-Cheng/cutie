/**
 * 组件引用关系分析工具
 *
 * 功能：
 * 1. 搜集所有组件的位置
 * 2. 阅读代码，编制引用关系图
 * 3. 找出所有不存在的文件和这些文件的实际目录
 * 4. 对所有的引用路径给出修改建议但不直接修改
 * 5. 正确处理重名
 */

import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'
import { dirname } from 'path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

interface ComponentInfo {
  name: string
  fullPath: string
  relativePath: string
  directory: string
}

interface ImportReference {
  importedName: string
  importPath: string
  importStatement: string
  lineNumber: number
  resolvedPath: string | null
  exists: boolean
}

interface FileAnalysis {
  filePath: string
  relativePath: string
  imports: ImportReference[]
  referencedBy: string[]
}

interface DuplicateComponent {
  name: string
  locations: string[]
}

interface PathSuggestion {
  file: string
  importLine: number
  currentPath: string
  issue: string
  suggestions: string[]
  confidence: 'high' | 'medium' | 'low'
}

class ComponentAnalyzer {
  private rootDir: string
  private srcDir: string
  private components: Map<string, ComponentInfo[]> = new Map()
  private fileAnalysis: Map<string, FileAnalysis> = new Map()
  private pathSuggestions: PathSuggestion[] = []

  constructor(rootDir: string) {
    this.rootDir = rootDir
    this.srcDir = path.join(rootDir, 'src')
  }

  /**
   * 步骤1: 搜集所有组件的位置
   */
  async collectAllComponents(): Promise<void> {
    console.log('📂 步骤 1: 搜集所有组件...\n')

    const files = this.getAllVueFiles(this.srcDir)

    for (const file of files) {
      const componentName = path.basename(file, '.vue')
      const relativePath = path.relative(this.srcDir, file).replace(/\\/g, '/')
      const directory = path.dirname(relativePath)

      const info: ComponentInfo = {
        name: componentName,
        fullPath: file,
        relativePath,
        directory,
      }

      if (!this.components.has(componentName)) {
        this.components.set(componentName, [])
      }
      this.components.get(componentName)!.push(info)
    }

    console.log(`✅ 找到 ${files.length} 个组件文件`)
    console.log(`✅ 识别 ${this.components.size} 个唯一组件名\n`)
  }

  /**
   * 步骤2: 分析所有文件的引用关系
   */
  async analyzeReferences(): Promise<void> {
    console.log('🔍 步骤 2: 分析引用关系...\n')

    const allFiles = [...this.getAllVueFiles(this.srcDir), ...this.getAllTsFiles(this.srcDir)]

    for (const file of allFiles) {
      const relativePath = path.relative(this.srcDir, file).replace(/\\/g, '/')
      const content = fs.readFileSync(file, 'utf-8')
      const imports = this.extractImports(file, content)

      this.fileAnalysis.set(file, {
        filePath: file,
        relativePath,
        imports,
        referencedBy: [],
      })
    }

    // 建立反向引用
    for (const [file, analysis] of this.fileAnalysis) {
      for (const imp of analysis.imports) {
        if (imp.resolvedPath && this.fileAnalysis.has(imp.resolvedPath)) {
          this.fileAnalysis.get(imp.resolvedPath)!.referencedBy.push(file)
        }
      }
    }

    console.log(`✅ 分析了 ${allFiles.length} 个文件的引用关系\n`)
  }

  /**
   * 步骤3: 找出不存在的文件引用
   */
  async findBrokenReferences(): Promise<void> {
    console.log('🔗 步骤 3: 检查断链引用...\n')

    let brokenCount = 0

    for (const [file, analysis] of this.fileAnalysis) {
      for (const imp of analysis.imports) {
        if (!imp.exists && imp.importPath.startsWith('.')) {
          brokenCount++
          await this.generateSuggestions(file, imp)
        }
      }
    }

    console.log(`✅ 找到 ${brokenCount} 个断链引用\n`)
  }

  /**
   * 步骤4: 生成修复建议
   */
  private async generateSuggestions(file: string, imp: ImportReference): Promise<void> {
    const suggestions: string[] = []
    const componentName = path.basename(imp.importPath, path.extname(imp.importPath))

    // 查找可能的匹配组件
    const possibleComponents = this.components.get(componentName)

    if (possibleComponents && possibleComponents.length > 0) {
      // 计算相对路径
      const fileDir = path.dirname(file)

      for (const component of possibleComponents) {
        let relativePath = path.relative(fileDir, component.fullPath)
        relativePath = relativePath.replace(/\\/g, '/')

        // 确保相对路径以 ./ 或 ../ 开头
        if (!relativePath.startsWith('.')) {
          relativePath = './' + relativePath
        }

        suggestions.push(relativePath)
      }

      this.pathSuggestions.push({
        file: path.relative(this.srcDir, file).replace(/\\/g, '/'),
        importLine: imp.lineNumber,
        currentPath: imp.importPath,
        issue: `文件不存在: ${imp.importPath}`,
        suggestions,
        confidence: possibleComponents.length === 1 ? 'high' : 'medium',
      })
    } else {
      // 尝试模糊匹配
      const fuzzyMatches = this.findFuzzyMatches(componentName)

      if (fuzzyMatches.length > 0) {
        const fileDir = path.dirname(file)

        for (const match of fuzzyMatches.slice(0, 3)) {
          let relativePath = path.relative(fileDir, match.fullPath)
          relativePath = relativePath.replace(/\\/g, '/')

          if (!relativePath.startsWith('.')) {
            relativePath = './' + relativePath
          }

          suggestions.push(relativePath)
        }

        this.pathSuggestions.push({
          file: path.relative(this.srcDir, file).replace(/\\/g, '/'),
          importLine: imp.lineNumber,
          currentPath: imp.importPath,
          issue: `文件不存在: ${imp.importPath} (可能的拼写错误)`,
          suggestions,
          confidence: 'low',
        })
      } else {
        this.pathSuggestions.push({
          file: path.relative(this.srcDir, file).replace(/\\/g, '/'),
          importLine: imp.lineNumber,
          currentPath: imp.importPath,
          issue: `文件不存在且未找到匹配: ${imp.importPath}`,
          suggestions: [],
          confidence: 'low',
        })
      }
    }
  }

  /**
   * 步骤5: 处理重名组件
   */
  findDuplicateComponents(): DuplicateComponent[] {
    console.log('👥 步骤 5: 检查重名组件...\n')

    const duplicates: DuplicateComponent[] = []

    for (const [name, locations] of this.components) {
      if (locations.length > 1) {
        duplicates.push({
          name,
          locations: locations.map((l) => l.relativePath),
        })
      }
    }

    console.log(`✅ 找到 ${duplicates.length} 个重名组件\n`)
    return duplicates
  }

  /**
   * 直接打印报告到命令行
   */
  printReport(): void {
    console.log('👥 检查重名组件...\n')
    const duplicates: DuplicateComponent[] = []
    for (const [name, locations] of this.components) {
      if (locations.length > 1) {
        duplicates.push({
          name,
          locations: locations.map((l) => l.relativePath),
        })
      }
    }
    console.log(`✅ 找到 ${duplicates.length} 个重名组件\n`)

    // 1. 组件统计
    console.log('📊 组件统计\n')
    console.log(`   总组件数: ${Array.from(this.components.values()).flat().length}`)
    console.log(`   唯一组件名: ${this.components.size}`)
    console.log(`   重名组件: ${duplicates.length}\n`)

    // 2. 重名组件详情
    if (duplicates.length > 0) {
      console.log('⚠️  重名组件列表\n')
      for (const dup of duplicates) {
        console.log(`   ${dup.name}`)
        for (const loc of dup.locations) {
          console.log(`      - ${loc}`)
        }
        console.log()
      }
    }

    // 3. 断链引用和修复建议
    if (this.pathSuggestions.length > 0) {
      console.log(`🔗 断链引用 (${this.pathSuggestions.length} 个)\n`)

      const highConfidence = this.pathSuggestions.filter((s) => s.confidence === 'high')
      const mediumConfidence = this.pathSuggestions.filter((s) => s.confidence === 'medium')
      const lowConfidence = this.pathSuggestions.filter((s) => s.confidence === 'low')

      if (highConfidence.length > 0) {
        console.log('   ✅ 高可信度修复建议:\n')
        for (const suggestion of highConfidence) {
          this.printSuggestion(suggestion)
        }
      }

      if (mediumConfidence.length > 0) {
        console.log('   ⚠️  中等可信度修复建议:\n')
        for (const suggestion of mediumConfidence) {
          this.printSuggestion(suggestion)
        }
      }

      if (lowConfidence.length > 0) {
        console.log('   ❓ 低可信度修复建议:\n')
        for (const suggestion of lowConfidence) {
          this.printSuggestion(suggestion)
        }
      }
    } else {
      console.log('✅ 断链引用检查\n')
      console.log('   未发现断链引用，所有导入路径都是有效的！\n')
    }

    // 4. 引用关系统计
    console.log('📈 引用关系统计\n')

    const mostReferenced = Array.from(this.fileAnalysis.values())
      .filter((a) => a.referencedBy.length > 0)
      .sort((a, b) => b.referencedBy.length - a.referencedBy.length)
      .slice(0, 10)

    if (mostReferenced.length > 0) {
      console.log('   被引用最多的文件 (Top 10):\n')
      for (const file of mostReferenced) {
        console.log(`      ${file.relativePath} - 被引用 ${file.referencedBy.length} 次`)
      }
      console.log()
    }

    const unused = Array.from(this.fileAnalysis.values()).filter(
      (a) => a.relativePath.includes('components/') && a.referencedBy.length === 0
    )

    if (unused.length > 0) {
      console.log(`   ⚠️  未被引用的组件 (${unused.length} 个):\n`)
      for (const file of unused) {
        console.log(`      - ${file.relativePath}`)
      }
      console.log()
    } else {
      console.log('   ✅ 所有组件都在使用中！\n')
    }

    // 5. 组件目录结构
    console.log('📁 组件目录结构\n')
    const dirStats = this.getDirectoryStats()
    const sortedDirs = Object.entries(dirStats).sort(([, a], [, b]) => b - a)
    for (const [dir, count] of sortedDirs) {
      console.log(`   ${dir}: ${count} 个组件`)
    }
    console.log()
  }

  private printSuggestion(suggestion: PathSuggestion): void {
    console.log(`      📍 ${suggestion.file}:${suggestion.lineNumber}`)
    console.log(`         问题: ${suggestion.issue}`)
    console.log(`         当前路径: ${suggestion.currentPath}`)

    if (suggestion.suggestions.length > 0) {
      console.log(`         建议修改为:`)
      for (let i = 0; i < suggestion.suggestions.length; i++) {
        console.log(`            ${i + 1}. ${suggestion.suggestions[i]}`)
      }
    } else {
      console.log(`         建议: 此文件可能已被删除，请检查是否需要移除此引用`)
    }
    console.log()
  }

  /**
   * 生成完整报告（已弃用，保留用于兼容）
   */
  generateReport(): string {
    const duplicates = this.findDuplicateComponents()

    let report = '# 组件引用关系分析报告\n\n'
    report += `生成时间: ${new Date().toLocaleString('zh-CN')}\n\n`
    report += '---\n\n'

    // 1. 组件统计
    report += '## 1. 组件统计\n\n'
    report += `- 总组件数: ${Array.from(this.components.values()).flat().length}\n`
    report += `- 唯一组件名: ${this.components.size}\n`
    report += `- 重名组件: ${duplicates.length}\n\n`

    // 2. 重名组件详情
    if (duplicates.length > 0) {
      report += '## 2. 重名组件列表\n\n'
      report += '⚠️ 以下组件存在多个同名文件，可能导致引用混淆：\n\n'

      for (const dup of duplicates) {
        report += `### ${dup.name}\n\n`
        for (const loc of dup.locations) {
          report += `- \`${loc}\`\n`
        }
        report += '\n'
      }
    }

    // 3. 断链引用和修复建议
    if (this.pathSuggestions.length > 0) {
      report += '## 3. 断链引用和修复建议\n\n'
      report += `找到 ${this.pathSuggestions.length} 个需要修复的引用：\n\n`

      // 按可信度分组
      const highConfidence = this.pathSuggestions.filter((s) => s.confidence === 'high')
      const mediumConfidence = this.pathSuggestions.filter((s) => s.confidence === 'medium')
      const lowConfidence = this.pathSuggestions.filter((s) => s.confidence === 'low')

      if (highConfidence.length > 0) {
        report += '### 3.1 高可信度修复建议 ✅\n\n'
        report += '这些建议有唯一匹配，可以安全修复：\n\n'
        for (const suggestion of highConfidence) {
          report += this.formatSuggestion(suggestion)
        }
      }

      if (mediumConfidence.length > 0) {
        report += '### 3.2 中等可信度修复建议 ⚠️\n\n'
        report += '这些引用有多个可能的匹配，需要手动确认：\n\n'
        for (const suggestion of mediumConfidence) {
          report += this.formatSuggestion(suggestion)
        }
      }

      if (lowConfidence.length > 0) {
        report += '### 3.3 低可信度修复建议 ❓\n\n'
        report += '这些引用可能是拼写错误或文件已删除：\n\n'
        for (const suggestion of lowConfidence) {
          report += this.formatSuggestion(suggestion)
        }
      }
    } else {
      report += '## 3. 断链引用检查\n\n'
      report += '✅ 未发现断链引用，所有导入路径都是有效的！\n\n'
    }

    // 4. 引用关系图
    report += '## 4. 引用关系统计\n\n'

    const mostReferenced = Array.from(this.fileAnalysis.values())
      .filter((a) => a.referencedBy.length > 0)
      .sort((a, b) => b.referencedBy.length - a.referencedBy.length)
      .slice(0, 10)

    if (mostReferenced.length > 0) {
      report += '### 被引用最多的文件 (Top 10):\n\n'
      for (const file of mostReferenced) {
        report += `- \`${file.relativePath}\` - 被引用 ${file.referencedBy.length} 次\n`
      }
      report += '\n'
    }

    const unused = Array.from(this.fileAnalysis.values()).filter(
      (a) => a.relativePath.includes('components/') && a.referencedBy.length === 0
    )

    if (unused.length > 0) {
      report += '### 未被引用的组件:\n\n'
      for (const file of unused) {
        report += `- \`${file.relativePath}\`\n`
      }
      report += '\n'
    }

    // 5. 组件目录结构
    report += '## 5. 组件目录结构\n\n'
    const dirStats = this.getDirectoryStats()
    for (const [dir, count] of Object.entries(dirStats).sort(([, a], [, b]) => b - a)) {
      report += `- \`${dir}\`: ${count} 个组件\n`
    }

    return report
  }

  private formatSuggestion(suggestion: PathSuggestion): string {
    let result = `#### 📍 ${suggestion.file}:${suggestion.lineNumber}\n\n`
    result += `**问题:** ${suggestion.issue}\n\n`
    result += `**当前路径:** \`${suggestion.currentPath}\`\n\n`

    if (suggestion.suggestions.length > 0) {
      result += `**建议修改为:**\n\n`
      for (let i = 0; i < suggestion.suggestions.length; i++) {
        result += `${i + 1}. \`${suggestion.suggestions[i]}\`\n`
      }
    } else {
      result += `**建议:** 此文件可能已被删除，请检查是否需要移除此引用。\n`
    }

    result += '\n---\n\n'
    return result
  }

  private getDirectoryStats(): Record<string, number> {
    const stats: Record<string, number> = {}

    for (const components of this.components.values()) {
      for (const component of components) {
        const dir = component.directory || '(root)'
        stats[dir] = (stats[dir] || 0) + 1
      }
    }

    return stats
  }

  /**
   * 提取文件中的所有 import 语句
   */
  private extractImports(file: string, content: string): ImportReference[] {
    const imports: ImportReference[] = []
    const lines = content.split('\n')

    // 匹配各种 import 模式
    const patterns = [
      // import Foo from './Foo.vue'
      /import\s+(\w+)\s+from\s+['"]([^'"]+)['"]/g,
      // import { Foo } from './Foo'
      /import\s+\{([^}]+)\}\s+from\s+['"]([^'"]+)['"]/g,
      // import * as Foo from './Foo'
      /import\s+\*\s+as\s+(\w+)\s+from\s+['"]([^'"]+)['"]/g,
      // const Foo = defineAsyncComponent(() => import('./Foo.vue'))
      /defineAsyncComponent\s*\(\s*\(\s*\)\s*=>\s*import\s*\(\s*['"]([^'"]+)['"]\s*\)\s*\)/g,
    ]

    // 如果是 .vue 文件，还需要检测模板中的组件使用
    const isVueFile = file.endsWith('.vue')
    const templateComponentUsages = isVueFile
      ? this.extractTemplateComponents(content)
      : new Set<string>()

    lines.forEach((line, index) => {
      for (const pattern of patterns) {
        const regex = new RegExp(pattern)
        let match

        while ((match = regex.exec(line)) !== null) {
          let importedName = ''
          let importPath = ''

          if (match[2]) {
            // 标准 import
            importedName = match[1].trim()
            importPath = match[2]
          } else if (match[1]) {
            // defineAsyncComponent
            importPath = match[1]
            importedName = path.basename(importPath, path.extname(importPath))
          }

          if (importPath) {
            const resolvedPath = this.resolvePath(file, importPath)
            const exists = resolvedPath ? fs.existsSync(resolvedPath) : false

            imports.push({
              importedName,
              importPath,
              importStatement: line.trim(),
              lineNumber: index + 1,
              resolvedPath,
              exists,
            })
          }
        }
      }
    })

    // 对于在模板中使用但未找到 import 的组件，尝试查找
    if (isVueFile && templateComponentUsages.size > 0) {
      const importedNames = new Set(imports.map((imp) => imp.importedName))
      for (const componentName of templateComponentUsages) {
        if (!importedNames.has(componentName)) {
          // 组件在模板中使用但没有找到明确的 import
          // 可能是全局注册或通过 components 选项注册
          // 尝试在 components 目录中查找
          const possibleComponents = this.components.get(componentName)
          if (possibleComponents && possibleComponents.length > 0) {
            // 取第一个匹配的组件路径
            const targetComponent = possibleComponents[0]
            const fileDir = path.dirname(file)
            let relativePath = path.relative(fileDir, targetComponent.fullPath)
            relativePath = relativePath.replace(/\\/g, '/')
            if (!relativePath.startsWith('.')) {
              relativePath = './' + relativePath
            }

            imports.push({
              importedName: componentName,
              importPath: relativePath,
              importStatement: `// 在模板中使用: <${componentName}>`,
              lineNumber: 0,
              resolvedPath: targetComponent.fullPath,
              exists: true,
            })
          }
        }
      }
    }

    return imports
  }

  /**
   * 从 Vue 模板中提取使用的组件名
   */
  private extractTemplateComponents(content: string): Set<string> {
    const components = new Set<string>()

    // 提取 <template> 部分
    const templateMatch = content.match(/<template[^>]*>([\s\S]*?)<\/template>/i)
    if (!templateMatch) return components

    const templateContent = templateMatch[1]

    // 匹配所有自定义组件标签（大写字母开头或包含连字符）
    // 匹配 <ComponentName 或 <component-name
    const componentTagPattern = /<([A-Z][a-zA-Z0-9]*)/g
    let match

    while ((match = componentTagPattern.exec(templateContent)) !== null) {
      const componentName = match[1]
      // 排除 HTML 原生标签和一些特殊标签
      if (!this.isNativeHtmlTag(componentName)) {
        components.add(componentName)
      }
    }

    // 还要检查 component :is 的情况
    const dynamicComponentPattern = /<component[^>]+:is=['"]([^'"]+)['"]/g
    while ((match = dynamicComponentPattern.exec(templateContent)) !== null) {
      const componentName = match[1]
      // 如果是简单的组件名（不是变量）
      if (/^[A-Z][a-zA-Z0-9]*$/.test(componentName)) {
        components.add(componentName)
      }
    }

    return components
  }

  /**
   * 判断是否为原生 HTML 标签
   */
  private isNativeHtmlTag(tagName: string): boolean {
    const nativeTags = new Set([
      'Html',
      'Head',
      'Body',
      'Div',
      'Span',
      'A',
      'P',
      'Ul',
      'Li',
      'Table',
      'Tr',
      'Td',
      'Th',
      'Form',
      'Input',
      'Button',
      'Select',
      'Option',
      'Textarea',
      'Label',
      'Img',
      'Video',
      'Audio',
      'Canvas',
      'Svg',
      'Path',
      'Circle',
      'Rect',
      'Line',
      'Polygon',
      'Component', // Vue 内置
      'Transition',
      'TransitionGroup',
      'KeepAlive',
      'Teleport',
      'Suspense',
      'RouterView',
      'RouterLink',
    ])
    return nativeTags.has(tagName)
  }

  /**
   * 解析相对路径和别名路径
   */
  private resolvePath(fromFile: string, importPath: string): string | null {
    let resolved: string

    // 处理 @ 别名（指向 src 目录）
    if (importPath.startsWith('@/')) {
      const pathAfterAlias = importPath.substring(2) // 去掉 '@/'
      resolved = path.join(this.srcDir, pathAfterAlias)
    }
    // 处理相对路径
    else if (importPath.startsWith('.')) {
      const fileDir = path.dirname(fromFile)
      resolved = path.resolve(fileDir, importPath)
    }
    // 跳过 node_modules 等其他导入
    else {
      return null
    }

    // 尝试添加扩展名
    const extensions = ['.vue', '.ts', '.js', '.tsx', '.jsx', '']
    for (const ext of extensions) {
      const withExt = resolved + ext
      if (fs.existsSync(withExt)) {
        return withExt
      }
    }

    // 尝试 index 文件
    const indexPaths = [
      path.join(resolved, 'index.vue'),
      path.join(resolved, 'index.ts'),
      path.join(resolved, 'index.js'),
    ]

    for (const indexPath of indexPaths) {
      if (fs.existsSync(indexPath)) {
        return indexPath
      }
    }

    return null
  }

  /**
   * 模糊匹配组件名
   */
  private findFuzzyMatches(name: string): ComponentInfo[] {
    const matches: Array<{ component: ComponentInfo; score: number }> = []
    const lowerName = name.toLowerCase()

    for (const [componentName, components] of this.components) {
      const lowerComponentName = componentName.toLowerCase()

      // 计算相似度
      let score = 0

      if (lowerComponentName === lowerName) {
        score = 100
      } else if (lowerComponentName.includes(lowerName)) {
        score = 80
      } else if (lowerName.includes(lowerComponentName)) {
        score = 70
      } else {
        // Levenshtein 距离
        const distance = this.levenshteinDistance(lowerName, lowerComponentName)
        if (distance <= 3) {
          score = 60 - distance * 10
        }
      }

      if (score > 40) {
        for (const component of components) {
          matches.push({ component, score })
        }
      }
    }

    return matches.sort((a, b) => b.score - a.score).map((m) => m.component)
  }

  /**
   * 计算 Levenshtein 距离
   */
  private levenshteinDistance(a: string, b: string): number {
    const matrix: number[][] = []

    for (let i = 0; i <= b.length; i++) {
      matrix[i] = [i]
    }

    for (let j = 0; j <= a.length; j++) {
      matrix[0][j] = j
    }

    for (let i = 1; i <= b.length; i++) {
      for (let j = 1; j <= a.length; j++) {
        if (b.charAt(i - 1) === a.charAt(j - 1)) {
          matrix[i][j] = matrix[i - 1][j - 1]
        } else {
          matrix[i][j] = Math.min(
            matrix[i - 1][j - 1] + 1,
            matrix[i][j - 1] + 1,
            matrix[i - 1][j] + 1
          )
        }
      }
    }

    return matrix[b.length][a.length]
  }

  /**
   * 获取所有 .vue 文件
   */
  private getAllVueFiles(dir: string): string[] {
    const files: string[] = []

    const walk = (currentDir: string) => {
      const entries = fs.readdirSync(currentDir, { withFileTypes: true })

      for (const entry of entries) {
        const fullPath = path.join(currentDir, entry.name)

        if (entry.isDirectory() && entry.name !== 'node_modules') {
          walk(fullPath)
        } else if (entry.isFile() && entry.name.endsWith('.vue')) {
          files.push(fullPath)
        }
      }
    }

    walk(dir)
    return files
  }

  /**
   * 获取所有 .ts 文件
   */
  private getAllTsFiles(dir: string): string[] {
    const files: string[] = []

    const walk = (currentDir: string) => {
      const entries = fs.readdirSync(currentDir, { withFileTypes: true })

      for (const entry of entries) {
        const fullPath = path.join(currentDir, entry.name)

        if (entry.isDirectory() && entry.name !== 'node_modules') {
          walk(fullPath)
        } else if (entry.isFile() && (entry.name.endsWith('.ts') || entry.name.endsWith('.js'))) {
          files.push(fullPath)
        }
      }
    }

    walk(dir)
    return files
  }

  /**
   * 导出 JSON 格式的详细数据
   */
  exportJSON(): any {
    return {
      timestamp: new Date().toISOString(),
      components: Array.from(this.components.entries()).map(([name, locations]) => ({
        name,
        count: locations.length,
        locations: locations.map((l) => l.relativePath),
      })),
      fileAnalysis: Array.from(this.fileAnalysis.values()).map((fa) => ({
        file: fa.relativePath,
        imports: fa.imports.map((imp) => ({
          name: imp.importedName,
          path: imp.importPath,
          exists: imp.exists,
          line: imp.lineNumber,
        })),
        referencedBy: fa.referencedBy.map((f) => path.relative(this.srcDir, f).replace(/\\/g, '/')),
      })),
      suggestions: this.pathSuggestions,
    }
  }
}

/**
 * 主函数
 */
async function main() {
  const rootDir = process.argv[2] || process.cwd()

  console.log('╔═══════════════════════════════════════════════════╗')
  console.log('║     组件引用关系分析工具 v1.0                    ║')
  console.log('╚═══════════════════════════════════════════════════╝')
  console.log()
  console.log(`📁 分析目录: ${rootDir}\n`)
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n')

  const analyzer = new ComponentAnalyzer(rootDir)

  try {
    await analyzer.collectAllComponents()
    await analyzer.analyzeReferences()
    await analyzer.findBrokenReferences()

    // 直接输出报告到命令行
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n')
    analyzer.printReport()
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n')
    console.log('✨ 分析完成！\n')
  } catch (error) {
    console.error('❌ 分析过程中出现错误:', error)
    process.exit(1)
  }
}

// 运行主函数
main()

export { ComponentAnalyzer }
