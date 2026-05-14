/**
 * Library entity representing a collection storage
 */
export interface Library {
  id: number;
  name: string;
  module_type: 'novel' | 'epub' | 'music' | 'note' | 'bilibili' | 'weread';
  storage_path: string;
  description: string | null;
  is_default: boolean;
  created_at: number;
  last_accessed_at: number | null;
}

/**
 * Request to create a new library
 */
export interface CreateLibraryRequest {
  name: string;
  moduleType: 'novel' | 'epub' | 'music' | 'note' | 'bilibili' | 'weread';
  storagePath: string;
  description?: string;
}
