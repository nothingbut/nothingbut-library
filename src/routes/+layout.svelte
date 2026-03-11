<script lang="ts">
  import AppLayout from '$lib/components/AppLayout.svelte';
  import '../app.css';
</script>

<AppLayout>
  {#snippet children()}
    <slot />
  {/snippet}
</AppLayout>
