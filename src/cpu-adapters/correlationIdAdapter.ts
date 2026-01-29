/**
 * CorrelationId生成器适配器
 *
 * 桥接项目的CorrelationId生成器
 */

import { generateCorrelationId } from '@/infra/correlation/correlationId'
import type { ICorrelationIdGenerator } from 'front-cpu'

export const correlationIdAdapter: ICorrelationIdGenerator = {
  generate: () => generateCorrelationId(),
}
