import { defineUserConfig } from 'vuepress/cli'
import { viteBundler } from '@vuepress/bundler-vite'
import { hopeTheme } from "vuepress-theme-hope";

export default defineUserConfig({
  lang: 'en-US',

  title: 'RocksDBFusion',
  description: 'MRocksDBFusion cis',

  base: process.env.NODE_ENV === 'production' ? '/RocksDBFusion/' : '/',

  theme: hopeTheme({
    logo: '/app.png',

    iconAssets: "fontawesome",

    // Assuming GitHub. Can also be a full url.
    repo: "s00d/RocksDBFusion",
    // Customizing the header label
    // Defaults to "GitHub" / "GitLab" / "Gitee" / "Bitbucket" or "Source" depending on `repo`
    repoLabel: "GitHub",
    // Whether to display repo link, default is `true`
    repoDisplay: true,

    navbar: [
      {
        text: "Home",
        link: "/",
        icon: "fas fa-home",
      },
      {
        text: "Server Info",
        icon: "fas fa-server",
        path: "/server/",
        prefix: "/server/",
        collapsible: true,
        // defaults to false
        expanded: true,
        children: [
          "README.md",
          "install.md",
          "methods.md",
          "structure.md",
        ],
      },
      {
        text: "Clients  Info",
        icon: "fas fa-code",
        path: "/clients/",
        prefix: "/clients/",
        collapsible: true,
        // defaults to false
        expanded: true,
        children: [
          "README.md",
          "php.md",
          "node.md",
          "rust.md",
          "python.md",
          "go.md",
        ],
      },
      {
        text: "Viewer",
        link: "/viewer",
        icon: "fas fa-eye",
      },
      {
        text: "Changelog",
        link: "/changelog",
        icon: "fas fa-history",
      },
      {
        text: "Development",
        path: "/develop/",
        icon: "fas fa-tools",
        prefix: "/develop/",
        collapsible: true,
        // defaults to false
        expanded: true,
        children: [
          "README.md",
          "generator.md",
          "server.md",
        ],
      },
    ],

    plugins: {
      blog: true,
      mdEnhance: {
        // Enable figure
        figure: true,
        // Enable image lazyload
        imgLazyload: true,
        // Enable image mark
        imgMark: true,
        // Enable image size
        imgSize: true,
        mermaid: true,
      },
    },
  }),

  bundler: viteBundler(),
})
