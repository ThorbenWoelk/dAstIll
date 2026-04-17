<script lang="ts">
  interface Props {
    channelName: string;
    title: string;
  }

  let { channelName, title }: Props = $props();
</script>

<header class="header">
  <p class="kicker">{channelName}</p>
  <h1 class="title">{title}</h1>
</header>

<style>
  .header {
    margin-bottom: var(--space-md);
  }
  .kicker {
    margin: 0 0 var(--space-sm);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--soft-foreground);
  }
  .title {
    font-family: "Fraunces", serif;
    font-size: 30px;
    font-weight: 600;
    letter-spacing: -0.03em;
    line-height: 1.1;
    font-variation-settings: "opsz" 72;
    margin: 0;
  }

  @media (min-width: 640px) {
    .title {
      font-size: 44px;
    }
    .kicker {
      font-size: 11px;
    }
  }

  @media (min-width: 960px) {
    .header {
      margin-bottom: var(--space-sm);
    }
    .title {
      font-size: 34px;
      line-height: 1.08;
    }
    .kicker {
      margin-bottom: 6px;
    }
  }
</style>
