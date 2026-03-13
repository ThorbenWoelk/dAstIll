<script lang="ts">
  import { onMount } from "svelte";

  import { registerAppServiceWorker } from "$lib/pwa";

  onMount(() => {
    void registerAppServiceWorker();
  });
</script>
