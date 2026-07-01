import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { defineConfig, type DefaultTheme } from 'vitepress';
import { withMermaid } from 'vitepress-plugin-mermaid';

type CatalogueNode = {
  title: string;
  name: string;
  children?: CatalogueNode[];
};

type Catalogue = {
  items: CatalogueNode[];
};

const cataloguePath = resolve(__dirname, '../catalogue.json');
const catalogue = JSON.parse(readFileSync(cataloguePath, 'utf8')) as Catalogue;

function toItem(sectionName: string, page: CatalogueNode): DefaultTheme.SidebarItem {
  const suffix = page.name === 'index' ? '' : page.name;
  const normalized = suffix.length === 0 ? `/${sectionName}/` : `/${sectionName}/${suffix}`;
  return {
    text: page.title,
    link: normalized
  };
}

function buildSidebar(items: CatalogueNode[]): DefaultTheme.Sidebar {
  const sidebar: DefaultTheme.Sidebar = [
    {
      text: 'Home',
      items: [{ text: 'Overview', link: '/' }]
    }
  ];

  for (const item of items) {
    if (item.name === 'deep-dive') {
      for (const section of item.children ?? []) {
        sidebar.push({
          text: section.title,
          collapsed: false,
          items: (section.children ?? []).map((page) => toItem(section.name, page))
        });
      }
      continue;
    }

    sidebar.push({
      text: item.title,
      collapsed: false,
      items: (item.children ?? []).map((page) => toItem(item.name, page))
    });
  }

  return sidebar;
}

const sidebar = buildSidebar(catalogue.items);

export default withMermaid(
  defineConfig({
    title: 'Z00Z Deep Wiki',
    description: 'Source-cited architecture, onboarding, and subsystem documentation for the Z00Z workspace.',
    lang: 'en-US',
    srcDir: '.',
    cleanUrls: true,
    appearance: 'dark',
    ignoreDeadLinks: true,
    head: [
      [
        'link',
        {
          rel: 'preconnect',
          href: 'https://fonts.googleapis.com'
        }
      ],
      [
        'link',
        {
          rel: 'preconnect',
          href: 'https://fonts.gstatic.com',
          crossorigin: ''
        }
      ],
      [
        'link',
        {
          rel: 'stylesheet',
          href: 'https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;700&family=JetBrains+Mono:wght@400;600&display=swap'
        }
      ]
    ],
    vite: {
      optimizeDeps: {
        include: ['medium-zoom']
      }
    },
    themeConfig: {
      logo: '/logo.svg',
      nav: [
        { text: 'Home', link: '/' },
        { text: 'Onboarding', link: '/onboarding/' },
        { text: 'Workspace', link: '/01-getting-started/workspace-overview' },
        { text: 'Architecture', link: '/02-architecture/system-overview' },
        { text: 'LLM Context', link: '/llms.txt' }
      ],
      search: {
        provider: 'local'
      },
      outline: [2, 3],
      sidebar
    },
    mermaid: {
      theme: 'base',
      themeVariables: {
        background: '#FFFFFF',
        primaryColor: '#F3E5F5',
        primaryTextColor: '#4A148C',
        primaryBorderColor: '#8E24AA',
        lineColor: '#546E7A',
        secondaryColor: '#E3F2FD',
        secondaryTextColor: '#0D47A1',
        secondaryBorderColor: '#1E88E5',
        tertiaryColor: '#FFF3E0',
        tertiaryTextColor: '#E65100',
        tertiaryBorderColor: '#FB8C00',
        mainBkg: '#F3E5F5',
        nodeBorder: '#8E24AA',
        clusterBkg: '#ECEFF1',
        clusterBorder: '#546E7A',
        titleColor: '#263238',
        defaultLinkColor: '#546E7A',
        edgeLabelBackground: '#FFFFFF',
        actorBkg: '#E3F2FD',
        actorBorder: '#1E88E5',
        actorTextColor: '#0D47A1',
        actorLineColor: '#546E7A',
        signalColor: '#263238',
        signalTextColor: '#263238',
        labelBoxBkgColor: '#FFFFFF',
        labelTextColor: '#263238',
        noteBkgColor: '#E8F5E9',
        noteTextColor: '#1B5E20',
        activationBkgColor: '#FFF3E0',
        activationBorderColor: '#FB8C00',
        sectionBkgColor: '#F3E5F5',
        altSectionBkgColor: '#ECEFF1',
        gridColor: '#D0D7DE',
        cScale0: '#E3F2FD',
        cScale1: '#F3E5F5',
        cScale2: '#FFF3E0',
        cScale3: '#E8F5E9',
        cScale4: '#EDE7F6',
        pie1: '#1E88E5',
        pie2: '#8E24AA',
        pie3: '#FB8C00',
        pie4: '#43A047',
        pie5: '#5E35B1',
        pieSectionTextColor: '#263238',
        taskBkgColor: '#E3F2FD',
        taskTextColor: '#0D47A1',
        taskTextDarkColor: '#263238',
        taskBorderColor: '#1E88E5',
        activeTaskBkgColor: '#F3E5F5',
        activeTaskBorderColor: '#8E24AA',
        critBkgColor: '#FFE0E0',
        critBorderColor: '#D32F2F',
        doneTaskBkgColor: '#E8F5E9',
        doneTaskBorderColor: '#43A047'
      }
    }
  })
);
