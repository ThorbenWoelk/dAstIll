import DefaultTheme from "vitepress/theme";
import { h } from "vue";
import AppearanceSwitchA11y from "./components/AppearanceSwitchA11y.vue";
import MermaidDiagram from "./components/MermaidDiagram.vue";
import "./custom.css";

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component("MermaidDiagram", MermaidDiagram);
  },
  Layout: () => {
    return h(DefaultTheme.Layout, null, {
      "layout-top": () => h(AppearanceSwitchA11y),
    });
  },
};
