<script lang="ts">
  import { dev } from "$app/environment";
  import { onMount } from "svelte";

  import {
    registerAppServiceWorker,
    unregisterAppServiceWorkers,
  } from "$lib/pwa";
  import { isTauriRuntime } from "$lib/tauri-runtime";

  onMount(() => {
    if (dev || isTauriRuntime()) {
      void unregisterAppServiceWorkers();
      return;
    }

    void registerAppServiceWorker();
  });
</script>
