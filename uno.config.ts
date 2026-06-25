import {
  defineConfig,
  presetUno,
  presetAttributify,
  transformerDirectives,
} from "unocss";

export default defineConfig({
  presets: [presetUno(), presetAttributify()],
  transformers: [transformerDirectives()],
  theme: {
    colors: {
      brand: {
        50: "#eef6ff",
        100: "#d9eaff",
        200: "#bcd9ff",
        300: "#8ec1ff",
        400: "#599dff",
        500: "#3478f6",
        600: "#1f5ae0",
        700: "#1947b8",
        800: "#1a3e96",
        900: "#1b3877",
      },
    },
  },
});
