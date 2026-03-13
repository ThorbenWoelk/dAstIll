<script lang="ts">
  let {
    activeTab = "channels",
    onTabChange = () => {},
  }: {
    activeTab?: "channels" | "videos" | "content";
    onTabChange?: (tab: "channels" | "videos" | "content") => void;
  } = $props();
</script>

<nav class="mobile-tab-bar lg:hidden" aria-label="Panel navigation">
  <button
    type="button"
    class={`mobile-tab-item ${activeTab === "channels" ? "mobile-tab-item--active" : ""}`}
    onclick={() => onTabChange("channels")}
    aria-current={activeTab === "channels" ? "page" : undefined}
  >
    <svg
      class="h-5 w-5"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <rect x="3" y="3" width="6" height="18" rx="1.5" />
      <rect x="15" y="3" width="6" height="18" rx="1.5" />
      <rect x="9" y="3" width="6" height="18" rx="1.5" />
    </svg>
    <span>Channels</span>
  </button>
  <button
    type="button"
    class={`mobile-tab-item ${activeTab === "videos" ? "mobile-tab-item--active" : ""}`}
    onclick={() => onTabChange("videos")}
    aria-current={activeTab === "videos" ? "page" : undefined}
  >
    <svg
      class="h-5 w-5"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <polygon points="6,3 20,12 6,21" />
    </svg>
    <span>Videos</span>
  </button>
  <button
    type="button"
    class={`mobile-tab-item ${activeTab === "content" ? "mobile-tab-item--active" : ""}`}
    onclick={() => onTabChange("content")}
    aria-current={activeTab === "content" ? "page" : undefined}
  >
    <svg
      class="h-5 w-5"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" />
      <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />
    </svg>
    <span>Content</span>
  </button>
</nav>
