<script lang="ts">
  import { reportFailure } from '../api/diagnostics';
  import { isTauriRuntime, openDesktopLink } from '../api/runtime';
  import WorkflowHelpDemo from './WorkflowHelpDemo.svelte';
  import EntryRow, { type EntryRowTone } from './workspace/EntryRow.svelte';
  import { animeListTabs } from './workspace/tabs';

  const reviewTab = animeListTabs[0].label;
  const importSites = [
    { name: 'AniList', url: 'https://anilist.co/settings/import' },
    { name: 'MyAnimeList', url: 'https://myanimelist.net/import.php' }
  ];
  const statuses: { tone: EntryRowTone; label: string; detail: string }[] = [
    {
      tone: 'matched',
      label: 'Automatyczne',
      detail: 'Aplikacja wybrała tytuł'
    },
    {
      tone: 'review',
      label: 'Niepewne',
      detail: 'Są propozycje do wyboru'
    },
    {
      tone: 'unmatched',
      label: 'Brak kandydatów',
      detail: 'Wyszukaj samodzielnie'
    },
    { tone: 'info', label: 'Ręczne', detail: 'Twój wybór' },
    {
      tone: 'suppressed',
      label: 'Zastąpione',
      detail: 'Inny wpis przejął dopasowanie'
    },
    { tone: 'ignored', label: 'Ignorowane', detail: 'Pominięte w eksporcie' }
  ];

  let dialog: HTMLDialogElement;
  let isDemoOpen = $state(false);

  export function openHelp() {
    dialog.showModal();
    isDemoOpen = true;
  }

  function openImportPage(event: MouseEvent, url: string) {
    if (
      globalThis.shindenToAnilist?.openExternalUrl === undefined &&
      !isTauriRuntime()
    ) {
      return;
    }

    event.preventDefault();
    void openDesktopLink(url).catch((error) => {
      reportFailure('open_xml_import_page', error);
    });
  }
</script>

<dialog
  class="modal"
  aria-labelledby="workflow-help-title"
  bind:this={dialog}
  onclose={() => (isDemoOpen = dialog.open)}
>
  <div
    class="modal-box border-base-content/10 max-h-[calc(100dvh-1.5rem)] max-w-2xl border p-4 sm:p-6"
  >
    <div class="mb-4 flex items-center justify-between gap-3">
      <h2 id="workflow-help-title" class="text-xl font-bold">
        Jak przenieść listę?
      </h2>
      <button
        type="button"
        class="btn btn-ghost btn-square btn-sm text-muted"
        aria-label="Zamknij pomoc"
        onclick={() => dialog.close()}
      >
        <span class="icon-[lucide--x] size-5" aria-hidden="true"></span>
      </button>
    </div>

    <section aria-labelledby="workflow-review-title">
      <h3 id="workflow-review-title" class="text-sm font-bold">
        Zacznij od zakładki „{reviewTab}”
      </h3>
      {#if isDemoOpen}
        <WorkflowHelpDemo />
      {/if}

      <ul
        class="mt-2 grid gap-1 px-2 text-xs"
        aria-label="Oznaczenia kandydatów"
      >
        <li class="flex items-center gap-2">
          <span
            class="icon-[lucide--star] text-warning mx-0.5 size-5 shrink-0"
            aria-hidden="true"
          ></span>
          <p class="text-muted">
            <strong class="text-base-content">Najlepszy kandydat</strong> bez automatycznego
            wyboru.
          </p>
        </li>
        <li class="flex items-center gap-2">
          <span
            class="border-warning/55 bg-warning/20 h-4 w-6 shrink-0 rounded-sm border"
            aria-hidden="true"
          ></span>
          <p class="text-muted">
            <strong class="text-warning">Żółte tło</strong> oznacza, że kandydat jest
            już dopasowany do innego wpisu.
          </p>
        </li>
      </ul>
    </section>

    <section class="mt-4" aria-labelledby="workflow-indicators-title">
      <h3 id="workflow-indicators-title" class="mb-2 text-sm font-bold">
        Kolory wpisów
      </h3>
      <ul class="grid grid-cols-2 gap-1.5 sm:grid-cols-3">
        {#each statuses as status (status.tone)}
          <li class="bg-base-200 rounded-box min-w-0">
            <div inert aria-hidden="true">
              <EntryRow
                tone={status.tone}
                isSelected={false}
                ariaLabel={status.label}
                title={status.label}
                compact
                separated={false}
                onSelect={() => {}}
              >
                <p class="flex items-center gap-1 text-xs font-bold">
                  {#if status.tone === 'ignored'}
                    <span class="icon-[lucide--eye-off] size-3 shrink-0"></span>
                  {/if}
                  {status.label}
                </p>
                <p class="mt-0.5 text-xs opacity-80">{status.detail}</p>
              </EntryRow>
            </div>
            <span class="sr-only">{status.label}. {status.detail}.</span>
          </li>
        {/each}
      </ul>
    </section>

    <div
      class="border-base-content/10 mt-4 grid grid-cols-[minmax(5rem,1fr)_minmax(0,3fr)] items-center gap-3 border-t pt-4 text-sm"
    >
      <div
        class="border-base-content/10 bg-base-200 -ml-4 flex justify-end rounded-r-lg border-y border-r px-2 py-3 sm:-ml-6 sm:pr-4"
      >
        <span class="btn btn-error btn-sm pointer-events-none">Eksport</span>
      </div>
      <div class="grid gap-2">
        <p class="font-semibold">Wgraj zapisany plik XML na stronie importu.</p>
        <div class="flex flex-wrap gap-2">
          {#each importSites as site (site.url)}
            <a
              class="btn btn-outline btn-sm"
              href={site.url}
              target="_blank"
              rel="noopener noreferrer"
              onclick={(event) => openImportPage(event, site.url)}
              onauxclick={(event) => {
                if (event.button === 1) openImportPage(event, site.url);
              }}
            >
              {site.name}
              <span
                class="icon-[lucide--external-link] size-3.5"
                aria-hidden="true"
              ></span>
            </a>
          {/each}
        </div>
        <p class="text-muted text-xs">
          Wpisy bez dopasowania zostaną pominięte.
        </p>
      </div>
    </div>
  </div>
  <form method="dialog" class="modal-backdrop" aria-hidden="true">
    <button type="submit" tabindex="-1">Zamknij pomoc</button>
  </form>
</dialog>
