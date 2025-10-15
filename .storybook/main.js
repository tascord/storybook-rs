// Storybook configuration for Rust WASM components
// This integrates the Rust-generated stories with Storybook's JS API

export default {
  stories: ['../example/pkg/**/*.stories.@(js|jsx|ts|tsx)'],
  addons: [
    '@storybook/addon-links',
    '@storybook/addon-essentials',
    '@storybook/addon-interactions',
  ],
  framework: {
    name: '@storybook/web-components',
    options: {},
  },
  docs: {
    autodocs: 'tag',
  },
};
