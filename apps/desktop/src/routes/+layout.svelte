<script lang="ts">
  import '../app.css';
  import { themeStore } from '$lib/theme/theme.svelte';

  let { children } = $props();

  // Keep <html data-theme> in sync with the theme store (covers initial mount).
  $effect(() => {
    themeStore.setTheme(themeStore.theme);
  });
</script>

{@render children()}
