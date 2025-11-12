// Storybook configuration for Rust WASM components
// This integrates the Rust-generated stories with Storybook's JS API

export default {
  stories: [
    '../storybook/stories/**/*.stories.@(js|jsx|ts|tsx|mdx)',
    '../example/pkg/**/*.stories.@(js|jsx|ts|tsx)'
  ],

  addons: ['@storybook/addon-links', '@storybook/addon-docs'],

  framework: {
    name: '@storybook/web-components-vite',
    options: {},
  }
};
