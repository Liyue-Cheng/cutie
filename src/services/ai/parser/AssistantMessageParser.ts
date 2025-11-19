import {
  type ToolName,
  toolNames,
  type TextContent,
  type ToolUse,
  type ToolParamName,
  toolParamNames,
  type AssistantMessageContent,
} from '../shared/cutie-tools'
import { logger, LogTags } from '@/infra/logging'

/**
 * Parser for assistant messages. Maintains state between chunks
 * to avoid reprocessing the entire message on each update.
 */
export class AssistantMessageParser {
  private contentBlocks: AssistantMessageContent[] = []
  private currentTextContent: TextContent | undefined = undefined
  private currentTextContentStartIndex = 0
  private currentToolUse: ToolUse | undefined = undefined
  private currentToolUseStartIndex = 0
  private currentParamName: ToolParamName | undefined = undefined
  private currentParamValueStartIndex = 0
  private readonly MAX_ACCUMULATOR_SIZE = 1024 * 1024 // 1MB limit
  private readonly MAX_PARAM_LENGTH = 1024 * 100 // 100KB per parameter limit

  private accumulator = ''

  /**
   * Initialize a new AssistantMessageParser instance.
   */
  constructor() {
    this.reset()
  }

  /**
   * Reset the parser state.
   */
  public reset(): void {
    this.contentBlocks = []
    this.currentTextContent = undefined
    this.currentTextContentStartIndex = 0
    this.currentToolUse = undefined
    this.currentToolUseStartIndex = 0
    this.currentParamName = undefined
    this.currentParamValueStartIndex = 0
    this.accumulator = ''
  }

  /**
   * Returns the current parsed content blocks
   */
  public getContentBlocks(): AssistantMessageContent[] {
    // Return a shallow copy to prevent external mutation
    return this.contentBlocks.slice()
  }

  /**
   * Process a new chunk of text and update the parser state.
   * @param chunk The new chunk of text to process.
   */
  public processChunk(chunk: string): AssistantMessageContent[] {
    if (this.accumulator.length + chunk.length > this.MAX_ACCUMULATOR_SIZE) {
      throw new Error('Assistant message exceeds maximum allowed size')
    }
    // Store the current length of the accumulator before adding the new chunk
    const accumulatorStartLength = this.accumulator.length

    for (let i = 0; i < chunk.length; i++) {
      const char = chunk[i]
      this.accumulator += char
      const currentPosition = accumulatorStartLength + i

      // There should not be a param without a tool use.
      if (this.currentToolUse && this.currentParamName) {
        const currentParamValue = this.accumulator.slice(this.currentParamValueStartIndex)
        if (currentParamValue.length > this.MAX_PARAM_LENGTH) {
          // Reset to a safe state
          this.currentParamName = undefined
          this.currentParamValueStartIndex = 0
          continue
        }
        const paramClosingTag = `</${this.currentParamName}>`
        // Streamed param content: always write the currently accumulated value
        if (currentParamValue.endsWith(paramClosingTag)) {
          // End of param value.
          const paramValue = currentParamValue.slice(0, -paramClosingTag.length)
          this.currentToolUse.params[this.currentParamName] = paramValue.trim()
          this.currentParamName = undefined
          continue
        } else {
          // Partial param value is accumulating.
          // Write the currently accumulated param content in real time
          this.currentToolUse.params[this.currentParamName] = currentParamValue
          continue
        }
      }

      // No currentParamName.

      if (this.currentToolUse) {
        const currentToolValue = this.accumulator.slice(this.currentToolUseStartIndex)
        const toolUseClosingTag = `</${this.currentToolUse.name}>`
        if (currentToolValue.endsWith(toolUseClosingTag)) {
          // End of a tool use.
          this.currentToolUse.partial = false

          logger.debug(LogTags.AI_PARSER, '工具调用解析完成', {
            toolName: this.currentToolUse.name,
            params: this.currentToolUse.params,
          })

          this.currentToolUse = undefined
          continue
        } else {
          const possibleParamOpeningTags = toolParamNames.map((name) => `<${name}>`)
          for (const paramOpeningTag of possibleParamOpeningTags) {
            if (this.accumulator.endsWith(paramOpeningTag)) {
              // Start of a new parameter.
              const paramName = paramOpeningTag.slice(1, -1)
              if (!toolParamNames.includes(paramName as ToolParamName)) {
                // Handle invalid parameter name gracefully
                continue
              }
              this.currentParamName = paramName as ToolParamName
              this.currentParamValueStartIndex = this.accumulator.length
              break
            }
          }

          // There's no current param, and not starting a new param.
          // Partial tool value is accumulating.
          continue
        }
      }

      // No currentToolUse.

      let didStartToolUse = false
      const possibleToolUseOpeningTags = toolNames.map((name) => `<${name}>`)

      for (const toolUseOpeningTag of possibleToolUseOpeningTags) {
        if (this.accumulator.endsWith(toolUseOpeningTag)) {
          // Extract and validate the tool name
          const extractedToolName = toolUseOpeningTag.slice(1, -1)

          // Check if the extracted tool name is valid
          if (!toolNames.includes(extractedToolName as ToolName)) {
            // Invalid tool name, treat as plain text and continue
            continue
          }

          // Start of a new tool use.
          this.currentToolUse = {
            type: 'tool_use',
            name: extractedToolName as ToolName,
            params: {},
            partial: true,
          }

          logger.debug(LogTags.AI_PARSER, '检测到工具调用开始', {
            toolName: extractedToolName,
          })

          this.currentToolUseStartIndex = this.accumulator.length

          // This also indicates the end of the current text content.
          if (this.currentTextContent) {
            this.currentTextContent.partial = false

            // Remove the partially accumulated tool use tag from the
            // end of text (<tool).
            this.currentTextContent.content = this.currentTextContent.content
              .slice(0, -toolUseOpeningTag.slice(0, -1).length)
              .trim()

            // No need to push, currentTextContent is already in contentBlocks
            this.currentTextContent = undefined
          }

          // Immediately push new tool_use block as partial
          let idx = this.contentBlocks.findIndex((block) => block === this.currentToolUse)
          if (idx === -1) {
            this.contentBlocks.push(this.currentToolUse)
          }

          didStartToolUse = true
          break
        }
      }

      if (!didStartToolUse) {
        // No tool use, so it must be text either at the beginning or
        // between tools.
        if (this.currentTextContent === undefined) {
          // If this is the first chunk and we're at the beginning of processing,
          // set the start index to the current position in the accumulator
          this.currentTextContentStartIndex = currentPosition

          // Create a new text content block and add it to contentBlocks
          this.currentTextContent = {
            type: 'text',
            content: this.accumulator.slice(this.currentTextContentStartIndex).trim(),
            partial: true,
          }

          // Add the new text content to contentBlocks immediately
          // Ensures it appears in the UI right away
          this.contentBlocks.push(this.currentTextContent)
        } else {
          // Update the existing text content
          this.currentTextContent.content = this.accumulator
            .slice(this.currentTextContentStartIndex)
            .trim()
        }
      }
    }
    // Do not call finalizeContentBlocks() here.
    // Instead, update any partial blocks in the array and add new ones as they're completed.
    // This matches the behavior of the original parseAssistantMessage function.
    return this.getContentBlocks()
  }

  /**
   * Finalize any partial content blocks.
   * Should be called after processing the last chunk.
   */
  public finalizeContentBlocks(): void {
    // Mark all partial blocks as complete
    for (const block of this.contentBlocks) {
      if (block.partial) {
        block.partial = false
      }
      if (block.type === 'text' && typeof block.content === 'string') {
        block.content = block.content.trim()
      }
    }
  }
}
