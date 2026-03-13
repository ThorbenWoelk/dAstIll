<script setup lang="ts">
import { onBeforeUnmount, onMounted } from "vue";

const switchSelector = ".VPSwitchAppearance";

const syncAppearanceSwitch = (node: Element) => {
  if (!(node instanceof HTMLButtonElement) || !node.matches(switchSelector)) {
    return;
  }

  const label = node.getAttribute("title");

  if (!label) {
    return;
  }

  node.setAttribute("aria-label", label);
  node.removeAttribute("title");
};

const syncAppearanceSwitches = (root: ParentNode | Element) => {
  if (root instanceof Element) {
    syncAppearanceSwitch(root);
  }

  root.querySelectorAll(switchSelector).forEach((node) => {
    syncAppearanceSwitch(node);
  });
};

let observer: MutationObserver | undefined;

onMounted(() => {
  syncAppearanceSwitches(document.body);

  observer = new MutationObserver((mutations) => {
    mutations.forEach((mutation) => {
      if (
        mutation.type === "attributes" &&
        mutation.target instanceof Element
      ) {
        syncAppearanceSwitch(mutation.target);
        return;
      }

      mutation.addedNodes.forEach((node) => {
        if (node instanceof Element) {
          syncAppearanceSwitches(node);
        }
      });
    });
  });

  observer.observe(document.body, {
    subtree: true,
    childList: true,
    attributes: true,
    attributeFilter: ["title"],
  });
});

onBeforeUnmount(() => {
  observer?.disconnect();
});
</script>

<template></template>
