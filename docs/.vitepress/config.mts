import { defineConfig } from "vitepress";

export default defineConfig({
  title: "Neothesia",
  description: "Flashy Synthesia Like Software For Linux, Windows and MacOs",
  base: "/Neothesia/",
  themeConfig: {
    nav: [
      { text: "Home", link: "/" },
      { text: "How-to", link: "/pages/installation" },
    ],

    sidebar: [
      {
        text: "How-to",
        items: [
          { text: "Installation", link: "/pages/installation" },
          { text: "Shortcuts", link: "/pages/shortcuts" },
          { text: "Customization", link: "/pages/customization" },
          { text: "Video Encoding", link: "/pages/video-encoding" },
        ],
      },
    ],

    socialLinks: [
      { icon: "github", link: "https://github.com/PolyMeilex/Neothesia" },
    ],
  },
});
