import { defaultTheme } from '@vuepress/theme-default'
import { defineUserConfig } from 'vuepress/cli'
import { viteBundler } from '@vuepress/bundler-vite'
import { hopeTheme } from "vuepress-theme-hope";

export default defineUserConfig({
  lang: 'en-US',

  title: 'RocksDBFusion',
  description: 'MRocksDBFusion cis',

  base: process.env.NODE_ENV === 'production' ? '/RocksDBFusion/' : '/',


  theme: hopeTheme({
    logo: 'https://vuejs.press/images/hero.png',

    // Assuming GitHub. Can also be a full url.
    repo: "s00d/RocksDBFusion",
    // Customizing the header label
    // Defaults to "GitHub" / "GitLab" / "Gitee" / "Bitbucket" or "Source" depending on `repo`
    repoLabel: "GitHub",
    // Whether to display repo link, default is `true`
    repoDisplay: true,

    navbar: ['/', 'server/', 'viewer', 'php', 'node', 'rust', 'python'],

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
