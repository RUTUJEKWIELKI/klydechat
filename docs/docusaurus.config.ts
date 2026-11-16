import { themes as prismThemes } from 'prism-react-renderer';
import type { Config } from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import type { ScalarOptions } from '@scalar/docusaurus';

const config: Config = {
  title: 'Klyde Developers',
  tagline: 'Developer documentation for Klyde',
  favicon: 'https://klyde.chat/favicon-klyde.svg',

  future: {
    v4: true,
  },

  url: 'https://developers.klyde.chat',
  baseUrl: '/docs/',

  organizationName: 'klydechat',
  projectName: 'klydechat',

  onBrokenLinks: 'throw',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          routeBasePath: '/',
          sidebarPath: './sidebars.ts',
          editUrl:
            'https://github.com/klydechat/klydechat/tree/main/docs/',
        },
      } satisfies Preset.Options,
    ],
  ],

  plugins: [
    [
      '@scalar/docusaurus',
      {
        label: 'API Reference',
        route: '/api-reference',
        showNavLink: true,
        configuration: {
          url: 'https://klyde.chat/api/openapi.json',
        },
      } as ScalarOptions,
    ],
    [
      '@docusaurus/plugin-client-redirects',
      {
        fromExtensions: ['html', 'htm'],
        redirects: [
          // legacy docs website (klydechat/developer-wiki)
          {
            from: '/developers/api/reference.html',
            to: '/api-reference',
          },
          {
            from: '/contrib.html',
            to: '/developing/contrib',
          },
          {
            from: '/contrib',
            to: '/developing/contrib',
          },
        ],
      }
    ],
  ],

  themeConfig: {
    // image: 'img/docusaurus-social-card.jpg',
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      logo: {
        alt: 'Klyde for Developers',
        src: '/img/navbar.light.svg',
        srcDark: '/img/navbar.dark.svg'
      },
      items: [
        {
          type: 'doc',
          docId: 'index',
          label: 'Docs'
        },
        {
          href: 'https://github.com/klydechat',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Developers',
          items: [
            {
              label: 'Source Code',
              href: 'https://github.com/klydechat'
            },
            {
              label: 'Help Translate',
              href: 'https://translate.klyde.chat'
            },
          ],
        },
        {
          title: 'Team',
          items: [
            {
              label: 'About',
              href: 'https://klyde.chat/about'
            },
            {
              label: 'Blog and Changelogs',
              href: 'https://klyde.chat/updates'
            },
            {
              label: 'Contact',
              href: 'https://support.klyde.chat'
            },
          ],
        },
        {
          title: 'Klyde on Socials',
          items: [
            {
              label: 'Bluesky',
              href: 'https://bsky.app/profile/klyde.chat'
            },
            {
              label: 'Reddit',
              href: 'https://reddit.com/r/klydechat'
            },
            {
              label: 'Klyde Server',
              href: 'https://stt.gg/Testers'
            },
          ],
        },
        {
          title: 'Legal',
          items: [
            {
              label: 'Community Guidelines',
              href: 'https://klyde.chat/legal/community-guidelines'
            },
            {
              label: 'Terms of Service',
              href: 'https://klyde.chat/legal/terms'
            },
            {
              label: 'Privacy Policy',
              href: 'https://klyde.chat/legal/privacy'
            },
            {
              label: 'Imprint',
              href: 'https://klyde.chat/legal/imprint'
            },
          ],
        },
      ],
      copyright: `© Revolt Platforms Ltd, ${new Date().getFullYear()}`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
