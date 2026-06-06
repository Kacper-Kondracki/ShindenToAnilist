export type ProviderOption = {
  id: string;
  label: string;
  site: string;
  accent: string;
  disabled?: boolean;
};

export const providers = [
  {
    id: "shinden",
    label: "Shinden",
    site: "shinden.pl",
    accent: "var(--ctp-mocha-mauve)",
    disabled: false,
  },
  {
    id: "ogladaj-anime",
    label: "Oglądaj Anime",
    site: "ogladajanime.pl",
    accent: "var(--ctp-mocha-sky)",
    disabled: false,
  },
  {
    id: "anime-zone",
    label: "AnimeZone",
    site: "animezone.pl",
    accent: "var(--ctp-mocha-red)",
    disabled: false,
  },
] as const satisfies readonly ProviderOption[];

export type Provider = (typeof providers)[number]["id"];

export function providerById(providerId: Provider) {
  return providers.find(({ id }) => id === providerId) ?? providers[0];
}
