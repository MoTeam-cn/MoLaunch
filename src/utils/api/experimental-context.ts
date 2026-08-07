import { experimentalManager } from './experimental'
import { EXPERIMENTAL_ACTIONS } from './experimental-actions'

export interface CollectContextResult {
  kind: string
  text: string
}

export interface CollectContextParams {
  kind: string
  versionId?: string
  conversationId?: number
}

export function experimentalListInstalledVersions(): Promise<string[]> {
  return experimentalManager<string[]>(EXPERIMENTAL_ACTIONS.LIST_INSTALLED_VERSIONS)
}

export function experimentalCollectContext(params: CollectContextParams): Promise<CollectContextResult> {
  return experimentalManager<CollectContextResult>(EXPERIMENTAL_ACTIONS.COLLECT_CONTEXT, params)
}
