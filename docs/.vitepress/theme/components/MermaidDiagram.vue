<script setup lang="ts">
import mermaid from "mermaid";
import { useData } from "vitepress";
import { computed, onMounted, ref, useSlots, watch, type VNode } from "vue";

const props = defineProps<{
  chart?: string;
  caption?: string;
}>();

const { isDark } = useData();
const slots = useSlots();

const svg = ref("");
const error = ref<string | null>(null);
const mounted = ref(false);
let renderVersion = 0;

async function waitForFonts() {
  if (typeof document === "undefined" || !("fonts" in document)) return;

  try {
    await document.fonts.ready;
  } catch {
    // If the browser does not fully support the FontFaceSet promise,
    // continue with rendering instead of blocking the docs page.
  }
}

function collectSlotText(nodes: VNode[] | undefined): string {
  if (!nodes) return "";

  return nodes
    .map((node) => {
      if (typeof node.children === "string") {
        return node.children;
      }
      if (Array.isArray(node.children)) {
        return collectSlotText(node.children as VNode[]);
      }
      return "";
    })
    .join("");
}

const chartSource = computed(() =>
  (props.chart ?? collectSlotText(slots.default?.())).trim(),
);

async function renderDiagram() {
  const source = chartSource.value;
  if (!source) {
    svg.value = "";
    error.value = null;
    return;
  }

  const version = ++renderVersion;

  await waitForFonts();

  mermaid.initialize({
    startOnLoad: false,
    securityLevel: "strict",
    theme: isDark.value ? "dark" : "neutral",
    fontFamily: '"Avenir Next", "Helvetica Neue", "Segoe UI", sans-serif',
    fontSize: 16,
    htmlLabels: true,
    flowchart: {
      useMaxWidth: false,
    },
    sequence: {
      useMaxWidth: false,
      wrap: true,
    },
  });

  try {
    const { svg: renderedSvg } = await mermaid.render(
      `dastill-docs-diagram-${version}`,
      source,
    );

    if (version !== renderVersion) return;

    svg.value = renderedSvg;
    error.value = null;
  } catch (err) {
    if (version !== renderVersion) return;

    svg.value = "";
    error.value =
      err instanceof Error ? err.message : "Unknown Mermaid render error.";
  }
}

onMounted(async () => {
  mounted.value = true;
  await renderDiagram();
});

watch([() => chartSource.value, () => isDark.value], async () => {
  if (!mounted.value) return;
  await renderDiagram();
});
</script>

<template>
  <figure class="mermaid-diagram-block">
    <div v-if="svg" class="mermaid-diagram" v-html="svg" />
    <div
      v-else-if="error"
      class="mermaid-diagram mermaid-diagram-error"
      role="alert"
    >
      <strong>Diagram failed to render.</strong>
      <span>{{ error }}</span>
    </div>
    <figcaption v-if="caption" class="mermaid-diagram-caption">
      {{ caption }}
    </figcaption>
  </figure>
</template>
