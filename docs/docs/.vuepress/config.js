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
        text: "Server",
        link: "/server/",
        icon: "fas fa-server",
      },
      {
        text: "Viewer",
        link: "/viewer",
        icon: "fas fa-eye",
      },
      {
        text: "Clients",
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
        text: "Develop",
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
      }
    ],

    plugins: {
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
