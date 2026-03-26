<script lang="ts">
  import { onMount } from 'svelte';
  import { appDataDir } from '@tauri-apps/api/path';
  import AppLayout from '$lib/components/AppLayout.svelte';
  import { setCurrentWorkspace } from '$lib/stores/workspace';
  import '../app.css';

  // Initialize default workspace on app startup
  onMount(async () => {
    try {
      // Get application data directory
      const appData = await appDataDir();
      // Ensure proper path separator
      const workspacePath = appData.endsWith('/')
        ? `${appData}workspace`
        : `${appData}/workspace`;

      const defaultWorkspace = {
        id: 'default',
        name: 'Default Workspace',
        path: workspacePath,
        createdAt: new Date(),
      };
      setCurrentWorkspace(defaultWorkspace);

      console.log('Workspace initialized:', workspacePath);
    } catch (error) {
      console.error('Failed to initialize workspace:', error);

      // Fallback to hardcoded path for development
      const fallbackWorkspace = {
        id: 'default',
        name: 'Default Workspace',
        path: '/Users/shichang/Workspace/program/.worktrees/nothingbut-mvp/claude/nothingbut-library',
        createdAt: new Date(),
      };
      setCurrentWorkspace(fallbackWorkspace);
    }
  });
</script>

<AppLayout>
  {#snippet children()}
    <slot />
  {/snippet}
</AppLayout>
