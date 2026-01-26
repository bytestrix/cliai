# CLIAI Documentation Website

This directory contains the CLIAI documentation website built with [Docusaurus](https://docusaurus.io/).

## 🚀 Development

### Prerequisites
- Node.js 18+
- npm

### Local Development
```bash
cd website
npm install
npm start
```

This starts a local development server and opens a browser window. Most changes are reflected live without having to restart the server.

### Build
```bash
npm run build
```

This command generates static content into the `build` directory and can be served using any static contents hosting service.

### Deployment
The documentation is automatically deployed to GitHub Pages when changes are pushed to the `main` branch.

## 📁 Structure

```
website/
├── docs/                   # Documentation pages
│   ├── intro.md           # Getting started
│   ├── installation.md    # Installation guide
│   ├── configuration.md   # Configuration
│   ├── usage.md          # Usage guide
│   ├── safety.md         # Safety & security
│   ├── troubleshooting.md # Troubleshooting
│   ├── architecture.md   # Architecture details
│   └── distribution.md   # Distribution guide
├── blog/                  # Blog posts (optional)
├── src/
│   ├── components/        # React components
│   ├── css/              # Custom CSS
│   └── pages/            # Custom pages
├── static/               # Static assets
├── docusaurus.config.ts  # Site configuration
└── sidebars.ts          # Sidebar configuration
```

## 🎨 Customization

### Adding New Pages
1. Create a new `.md` file in the `docs/` directory
2. Add front matter with `sidebar_position`
3. Update `sidebars.ts` if needed

### Modifying the Homepage
Edit `src/pages/index.tsx` and `src/components/HomepageFeatures/index.tsx`

### Custom Styling
Add CSS to `src/css/custom.css` or create component-specific CSS modules

## 📝 Writing Documentation

### Front Matter
Each documentation page should include front matter:

```markdown
---
sidebar_position: 1
title: Page Title
---

# Page Content
```

### Code Blocks
Use syntax highlighting for code examples:

````markdown
```bash
cliai "example command"
```
````

### Admonitions
Use admonitions for important information:

```markdown
:::tip
This is a helpful tip!
:::

:::warning
This is a warning!
:::

:::danger
This is dangerous!
:::
```

## 🔗 Links

- **Live Site**: https://cliai-team.github.io/cliai/
- **Docusaurus Docs**: https://docusaurus.io/docs
- **Main Repository**: https://github.com/cliai-team/cliai