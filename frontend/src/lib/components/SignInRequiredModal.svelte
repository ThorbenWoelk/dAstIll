<script lang="ts">
  import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";

  /** Shown when a signed-in account is required (anonymous user attempted a protected action). */
  let {
    show = false,
    message = "Sign in to continue.",
    onConfirm,
    onCancel,
  }: {
    show?: boolean;
    message?: string;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();
</script>

<ConfirmationModal
  {show}
  title="Sign in required"
  {message}
  confirmLabel="Sign in"
  cancelLabel="Not now"
  tone="info"
  {onConfirm}
  {onCancel}
/>
