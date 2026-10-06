<script lang="ts">
  let { message, onDismiss }: { message: string; onDismiss: () => void } =
    $props();
</script>

<div class="notice" role="status">
  <p>{message}</p>
  <button type="button" class="text-button" onclick={onDismiss}>Dismiss</button>
</div>

<style>
  .notice {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 0 var(--space-4);
    margin-top: var(--space-4);
    padding: 0 var(--space-4);
    background: var(--paper-raised);
    border-left: 2px solid var(--kicker);
  }

  p {
    margin: var(--space-3) 0;
    font-family: var(--sans);
    font-size: 13px;
  }
</style>
