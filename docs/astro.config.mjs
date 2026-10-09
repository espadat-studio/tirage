import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import starlightLlmsTxt from "starlight-llms-txt";

export default defineConfig({
  site: "https://tirage.espadat.com",
  integrations: [
    starlight({
      plugins: [starlightLlmsTxt()],
      title: "tirage",
      description: "Original generative artwork and animations, rendered from code.",
      logo: { src: "./src/assets/mark.svg", alt: "Espadat" },
      head: [{ tag: "link", attrs: { rel: "icon", href: "/favicon.ico", sizes: "16x16 32x32" } }],
      customCss: ["@espadat/docs-theme/styles/theme.css"],
      components: {
        Footer: "@espadat/docs-theme/components/footer.astro",
        ThemeProvider: "@espadat/docs-theme/components/theme-provider.astro",
        ThemeSelect: "@espadat/docs-theme/components/theme-select.astro",
      },
      social: [
        { icon: "github", label: "GitHub", href: "https://github.com/espadat-studio/tirage" },
      ],
      sidebar: [
        { label: "Home", link: "/" },
        {
          label: "Getting Started",
          items: [{ slug: "getting-started/installation" }, { slug: "getting-started/quick-start" }],
        },
        {
          label: "Concepts",
          items: [
            { slug: "concepts/seed-and-recipe" },
            { slug: "concepts/pins" },
            { slug: "concepts/taste-bounds" },
            { slug: "concepts/still-and-loop" },
            { slug: "concepts/reproducing-the-look" },
          ],
        },
        {
          label: "Tools",
          items: [
            { label: "Overview", slug: "tools" },
            { slug: "tools/sonar" },
            { slug: "tools/husk" },
            { slug: "tools/vein" },
            { slug: "tools/aura" },
            { slug: "tools/kiosk" },
            { slug: "tools/frond" },
          ],
        },
        {
          label: "CLI Reference",
          items: [
            { slug: "cli-reference/tirage" },
            { slug: "cli-reference/derive" },
            { slug: "cli-reference/render" },
            { slug: "cli-reference/tools" },
            { slug: "cli-reference/completions" },
          ],
        },
      ],
    }),
  ],
});
