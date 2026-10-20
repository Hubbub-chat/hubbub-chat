import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { WorkspaceDocument } from '../types';

/** Loads (or full-text searches) workspace documents; reloads when `refreshKey` changes. */
export function useWorkspaceDocuments(query: string, refreshKey = 0) {
  const [documents, setDocuments] = useState<WorkspaceDocument[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const reload = useCallback(async () => {
    setIsLoading(true);
    try {
      const q = query.trim();
      const docs = q
        ? await invoke<WorkspaceDocument[]>('search_documents', { query: q })
        : await invoke<WorkspaceDocument[]>('list_documents');
      setDocuments(docs);
    } catch (err) {
      console.error('Failed to load documents:', err);
    } finally {
      setIsLoading(false);
    }
  }, [query]);

  useEffect(() => {
    // Debounce typing so full-text search isn't fired on every keystroke.
    const timer = setTimeout(reload, query ? 250 : 0);
    return () => clearTimeout(timer);
  }, [reload, refreshKey, query]);

  return { documents, isLoading, reload };
}
