export default {
  plugins: ["prettier-plugin-svelte"],

  semi: true,
  singleQuote: true,
  trailingComma: "none",

  overrides: [
    {
      files: "*.svelte",
      options: {
        parser: "svelte",
      },
    },
  ],
};
