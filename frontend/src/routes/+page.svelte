<script lang="ts">
  import { APP_MAINTENANCE_MODE } from "$lib/app-config";
  import MaintenancePage from "$lib/components/home/MaintenancePage.svelte";
  import WorkspaceHomePage from "$lib/components/home/WorkspaceHomePage.svelte";
</script>

{#if APP_MAINTENANCE_MODE}
  <MaintenancePage />
{:else}
  <WorkspaceHomePage />
{/if}
