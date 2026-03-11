<script lang="ts">
  import WorkspaceSelector from '$lib/components/WorkspaceSelector.svelte';
  import LibraryGrid from '$lib/components/LibraryGrid.svelte';
</script>

<div class="library-home">
  <WorkspaceSelector />
  <LibraryGrid />
</div>

<style>
  .library-home {
    display: flex;
    width: 100%;
    height: 100%;
    background-color: var(--color-bg-secondary);
  }
</style>
