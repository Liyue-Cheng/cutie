import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Tag } from '@/types/models'

type ID = string

export interface CreateTagPayload {
  title: string
  color?: string | null
  sort_key?: string | null
}

export interface UpdateTagPayload {
  title?: string
  color?: string | null
  sort_key?: string | null
}

export const useTagStore = defineStore('tag', () => {
  const tags = ref(new Map<ID, Tag>())
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const allTags = computed(() => {
    return Array.from(tags.value.values()).sort((a, b) => {
      if (a.sort_key && b.sort_key) {
        return a.sort_key.localeCompare(b.sort_key)
      }
      return a.title.localeCompare(b.title)
    })
  })

  function getTagById(id: ID): Tag | undefined {
    return tags.value.get(id)
  }

  async function fetchTags() {
    isLoading.value = true
    error.value = null
    try {
      const tagList = await invoke<Tag[]>('list_tags')
      const tagMap = new Map<ID, Tag>()
      for (const tag of tagList) {
        tagMap.set(tag.id, tag)
      }
      tags.value = tagMap
    } catch (e) {
      error.value = `Failed to fetch tags: ${e}`
      console.error(e)
    } finally {
      isLoading.value = false
    }
  }

  async function createTag(payload: CreateTagPayload) {
    isLoading.value = true
    error.value = null
    try {
      const newTag = await invoke<Tag>('create_tag', { payload })
      tags.value.set(newTag.id, newTag)
    } catch (e) {
      error.value = `Failed to create tag: ${e}`
      console.error(e)
    } finally {
      isLoading.value = false
    }
  }

  async function updateTag(id: ID, payload: UpdateTagPayload) {
    isLoading.value = true
    error.value = null
    try {
      const updatedTag = await invoke<Tag>('update_tag', { id, payload })
      tags.value.set(id, updatedTag)
    } catch (e) {
      error.value = `Failed to update tag ${id}: ${e}`
      console.error(e)
    } finally {
      isLoading.value = false
    }
  }

  async function deleteTag(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      await invoke('delete_tag', { id })
      tags.value.delete(id)
    } catch (e) {
      error.value = `Failed to delete tag ${id}: ${e}`
      console.error(e)
    } finally {
      isLoading.value = false
    }
  }

  return {
    tags,
    isLoading,
    error,
    allTags,
    getTagById,
    fetchTags,
    createTag,
    updateTag,
    deleteTag,
  }
})
