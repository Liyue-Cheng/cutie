/**
 * CorrelationId生成器适配器
 *
 * 桥接项目的CorrelationId生成器
 */

import { generateCorrelationId } from '@/infra/correlation/correlationId'
import type { ICorrelationIdGenerator } from '@cutie/cpu-pipeline'

export const correlationIdAdapter: ICorrelationIdGenerator = {
  generate: () => generateCorrelationId(),
}
