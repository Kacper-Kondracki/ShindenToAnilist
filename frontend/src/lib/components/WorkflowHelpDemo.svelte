<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { reportFailure } from '../api/diagnostics';
  import EntryRow, { type EntryRowTone } from './workspace/EntryRow.svelte';
  import RowMetadataBadges from './workspace/RowMetadataBadges.svelte';
  import { animeListTabs } from './workspace/tabs';

  const scenarios = [
    {
      id: 'choose',
      label: 'Wybór wyniku',
      description: 'Otwórz wpis i kliknij właściwego kandydata.',
      steps: 1
    },
    {
      id: 'reassign',
      label: 'Zajęty wynik',
      description:
        '„Wybierz mimo to” przenosi dopasowanie. Poprzedni wpis wraca do sprawdzenia.',
      steps: 2
    },
    {
      id: 'ignore',
      label: 'Ignorowanie',
      description:
        'Kliknij wpis prawym przyciskiem i wybierz „Ignoruj wpis”, aby pominąć go w eksporcie.',
      steps: 2
    },
    {
      id: 'search',
      label: 'Inny tytuł',
      description:
        'Wyniki są niejasne? Spróbuj japońskiego tytułu zapisanego alfabetem łacińskim.',
      steps: 4
    }
  ] as const;

  type Scenario = (typeof scenarios)[number];
  type ExampleAnime = { title: string; year: string; episodes: number };
  type RowOptions = {
    selected?: boolean;
    score?: string;
    star?: boolean;
    softWarning?: boolean;
    status?: string;
  };

  const mob = { title: 'Mob Psycho 100 II', year: '2019', episodes: 13 };
  const previousMob = { ...mob, title: 'Mob Psycho 100 2' };
  const kimetsu = { title: 'Kimetsu no Yaiba', year: '2019', episodes: 26 };
  const demonSlayer = { ...kimetsu, title: 'Demon Slayer' };
  const onigiri = { title: 'Onigiri', year: '2016', episodes: 13 };

  type Point = { x: number; y: number };

  let playback = $state.raw<{ scenario: Scenario; manuallySelected: boolean }>({
    scenario: scenarios[0],
    manuallySelected: false
  });
  let scenario = $derived(playback.scenario);
  let autoAdvance = $state(true);
  let autoAdvanceLabel = $derived(
    autoAdvance
      ? 'Wstrzymaj automatyczne przełączanie'
      : 'Wznów automatyczne przełączanie'
  );
  let step = $state(0);
  let reducedMotion = $state(false);
  let finished = $derived(step === scenario.steps);
  let typedTitle = $state('');
  let menuPosition = $state<Point>({ x: 0, y: 0 });
  let menuItemHovered = $state(false);
  let highlight = $state<(Point & { width: number; height: number }) | null>(
    null
  );
  let scenarioList: HTMLDivElement;
  let preview: HTMLDivElement;
  let pointer: HTMLSpanElement;
  let contextMenu = $state<HTMLUListElement | null>(null);

  onMount(() => {
    const preference = window.matchMedia('(prefers-reduced-motion: reduce)');

    function updateMotionPreference() {
      reducedMotion = preference.matches;
    }

    updateMotionPreference();
    preference.addEventListener('change', updateMotionPreference);
    const resizeObserver = new ResizeObserver(() =>
      positionHighlight(scenario.id)
    );
    resizeObserver.observe(scenarioList);

    return () => {
      preference.removeEventListener('change', updateMotionPreference);
      resizeObserver.disconnect();
    };
  });

  function positionHighlight(id: Scenario['id']) {
    const button = scenarioList.querySelector<HTMLButtonElement>(
      `[data-scenario="${id}"]`
    );
    if (button === null) return;

    highlight = {
      x: button.offsetLeft,
      y: button.offsetTop,
      width: button.offsetWidth,
      height: button.offsetHeight
    };
  }

  $effect(() => {
    const id = scenario.id;
    void tick().then(() => {
      if (scenarioList?.isConnected) positionHighlight(id);
    });
  });

  function play(nextScenario: Scenario, manuallySelected = true) {
    playback = { scenario: nextScenario, manuallySelected };
  }

  $effect(() => {
    const current = playback.scenario;
    const prefersStill = reducedMotion;
    step = prefersStill ? current.steps : 0;
    typedTitle = prefersStill ? kimetsu.title : '';
    menuItemHovered = false;

    if (prefersStill) return;

    const controller = new AbortController();
    void animateScenario(current.id, controller.signal).catch((error) => {
      if (controller.signal.aborted) return;
      reportFailure('workflow_help_animation', error);
      step = current.steps;
      typedTitle = kimetsu.title;
    });

    return () => controller.abort();
  });

  $effect(() => {
    if (!autoAdvance || reducedMotion || !finished) return;

    const { scenario: current, manuallySelected } = playback;
    const next = scenarios[scenarios.indexOf(current) + 1] ?? scenarios[0];
    const timeout = window.setTimeout(
      () => play(next, false),
      manuallySelected ? 5500 : 2200
    );

    return () => window.clearTimeout(timeout);
  });

  async function pause(duration: number, signal: AbortSignal) {
    await new Promise<void>((resolve) => window.setTimeout(resolve, duration));
    signal.throwIfAborted();
  }

  function translate(point: Point, scale = 1) {
    return `translate(${point.x}px, ${point.y}px) scale(${scale})`;
  }

  async function animateScenario(id: Scenario['id'], signal: AbortSignal) {
    await tick();
    signal.throwIfAborted();

    const cursor = pointer;
    let position = { x: preview.clientWidth * 0.26, y: 75 };
    cursor.style.transform = translate(position);
    cursor.style.opacity = '1';

    async function motion(
      keyframes: Keyframe[],
      options: KeyframeAnimationOptions
    ) {
      signal.throwIfAborted();
      const animation = cursor.animate(keyframes, options);
      const cancel = () => animation.cancel();
      signal.addEventListener('abort', cancel, { once: true });

      try {
        await animation.finished;
      } finally {
        signal.removeEventListener('abort', cancel);
      }
    }

    async function click(targetName: string, fraction = 0.46) {
      await tick();
      signal.throwIfAborted();
      const element = preview.querySelector<HTMLElement>(
        `[data-demo-target="${targetName}"]`
      );
      if (element === null)
        throw new Error(`Missing help action ${targetName}`);

      const frame = preview.getBoundingClientRect();
      const target = element.getBoundingClientRect();
      const scale = frame.width / preview.offsetWidth;
      const destination = {
        x:
          (target.left + target.width * fraction - frame.left) / scale -
          preview.clientLeft,
        y:
          (target.top + target.height * 0.53 - frame.top) / scale -
          preview.clientTop
      };
      const distance = Math.hypot(
        destination.x - position.x,
        destination.y - position.y
      );
      await motion(
        [
          { transform: translate(position), offset: 0 },
          {
            transform: translate({
              x: position.x + (destination.x - position.x) * 0.55,
              y: (position.y + destination.y) / 2 - Math.min(12, distance / 10)
            }),
            offset: 0.55
          },
          { transform: translate(destination), offset: 1 }
        ],
        {
          duration: Math.min(780, Math.max(300, distance * 1.8)),
          easing: 'ease-in-out'
        }
      );

      position = destination;
      cursor.style.transform = translate(position);
      if (targetName === 'ignore') {
        menuItemHovered = true;
        await pause(320, signal);
      }
      await motion(
        [
          { transform: translate(position) },
          { transform: translate(position, 0.74) },
          { transform: translate(position) }
        ],
        { duration: 160, easing: 'ease-in-out' }
      );
      signal.throwIfAborted();
    }

    try {
      await pause(350, signal);

      switch (id) {
        case 'choose':
          await click('candidate');
          step = 1;
          break;
        case 'reassign':
          await click('candidate');
          step = 1;
          await pause(1400, signal);
          await click('confirm');
          step = 2;
          break;
        case 'ignore':
          await click('source', 0.39);
          menuPosition = position;
          step = 1;
          await tick();
          signal.throwIfAborted();
          if (contextMenu === null)
            throw new Error('Missing help context menu');
          menuPosition = {
            x: Math.max(
              4,
              Math.min(
                position.x,
                preview.clientWidth - contextMenu.offsetWidth - 4
              )
            ),
            y: Math.max(
              4,
              Math.min(
                position.y,
                preview.clientHeight - contextMenu.offsetHeight - 4
              )
            )
          };
          await pause(1050, signal);
          await click('ignore');
          step = 2;
          break;
        case 'search':
          await click('search', 0.34);
          step = 1;
          await pause(550, signal);
          step = 2;
          for (const [index, letter] of Array.from(kimetsu.title).entries()) {
            await pause(index % 4 === 0 ? 110 : 65, signal);
            typedTitle += letter;
          }
          step = 3;
          await pause(900, signal);
          await click('candidate');
          step = 4;
          break;
      }

      await pause(400, signal);
    } finally {
      if (!signal.aborted) cursor.style.opacity = '0';
    }
  }
</script>

<div class="demo__toolbar mt-2">
  <div
    class="demo__scenario-list flex flex-wrap justify-center gap-1 max-[28rem]:grid max-[28rem]:grid-cols-2"
    role="group"
    aria-label="Przykłady dopasowania"
    bind:this={scenarioList}
  >
    {#if highlight !== null}
      <span
        class="demo__highlight"
        aria-hidden="true"
        style:transform={translate(highlight)}
        style:width={`${highlight.width}px`}
        style:height={`${highlight.height}px`}
      ></span>
    {/if}
    {#each scenarios as example (example.id)}
      <button
        type="button"
        class="btn btn-ghost btn-xs"
        data-scenario={example.id}
        aria-pressed={scenario.id === example.id}
        onclick={() => play(example)}
      >
        {example.label}
      </button>
    {/each}
  </div>
  <div
    class="demo__controls badge border-base-content/15 bg-base-200 h-auto gap-0.5 rounded-md p-0.5"
    class:invisible={reducedMotion}
  >
    <button
      type="button"
      class="btn btn-ghost btn-square btn-xs hover:bg-info/20 size-7 rounded-sm"
      disabled={reducedMotion}
      aria-label="Odtwórz przykład ponownie"
      title="Odtwórz przykład ponownie"
      onclick={() => play(scenario)}
    >
      <span
        class="icon-[lucide--rotate-ccw] text-info size-4"
        aria-hidden="true"
      ></span>
    </button>
    <button
      type="button"
      class={`btn btn-ghost btn-square btn-xs size-7 rounded-sm ${autoAdvance ? 'hover:bg-warning/20' : 'hover:bg-success/20'}`}
      disabled={reducedMotion}
      aria-label={autoAdvanceLabel}
      title={autoAdvanceLabel}
      onclick={() => (autoAdvance = !autoAdvance)}
    >
      <span
        class={`size-4 ${autoAdvance ? 'icon-[lucide--pause] text-warning' : 'icon-[lucide--play] text-success'}`}
        aria-hidden="true"
      ></span>
    </button>
  </div>
</div>

<p
  class="demo__description text-muted mt-1 text-sm"
  aria-live={autoAdvance && !reducedMotion && !playback.manuallySelected
    ? 'off'
    : 'polite'}
>
  {#each scenarios as example (example.id)}
    <span
      class:invisible={scenario.id !== example.id}
      aria-hidden={scenario.id !== example.id}
    >
      {example.description}
    </span>
  {/each}
  {#if scenario.id === 'search'}
    <span class="sr-only">
      Zmień zapytanie z {demonSlayer.title} na {kimetsu.title}.
    </span>
  {/if}
</p>

<div class="demo mt-2" inert aria-hidden="true" bind:this={preview}>
  <div class="demo__pane">
    <div class="demo__label">
      {scenario.id === 'reassign'
        ? animeListTabs[3].label
        : animeListTabs[0].label}
    </div>
    <div class="demo__rows">
      {#if scenario.id === 'ignore'}
        <div data-demo-target="source">
          {@render exampleRow(mob, finished ? 'ignored' : 'unmatched', {
            selected: true
          })}
        </div>
      {:else if scenario.id === 'search'}
        {@render exampleRow(demonSlayer, finished ? 'info' : 'review', {
          selected: true
        })}
      {:else}
        {@render exampleRow(mob, finished ? 'info' : 'review', {
          selected: true
        })}
        {#if scenario.id === 'reassign'}
          {@render exampleRow(
            previousMob,
            finished ? 'suppressed' : 'matched',
            {
              status: finished ? 'Zastąpione' : 'Automatyczne'
            }
          )}
        {/if}
      {/if}
    </div>
  </div>

  <div class="demo__pane">
    <div class="demo__label">
      {scenario.id === 'search'
        ? 'Wyniki wyszukiwania'
        : scenario.id === 'ignore'
          ? 'Dopasowanie'
          : 'Najlepsze kandydaty'}
    </div>
    <div class="demo__rows">
      {#if scenario.id === 'ignore'}
        <div class="flex items-center gap-2">
          <span class="input input-sm min-w-0 flex-1 truncate">
            {mob.title}
          </span>
          <span
            class="btn btn-neutral btn-soft btn-square btn-sm border-0"
            class:btn-active={finished}
          >
            <span class="icon-[lucide--eye-off] size-4"></span>
          </span>
        </div>
        <p class="text-muted px-1 text-xs">Brak wyników</p>
      {:else if scenario.id === 'search'}
        <div
          class="input input-sm demo__input w-full"
          data-demo-target="search"
          class:demo__input--focused={step === 1 || step === 2}
        >
          <span class:demo__selection={step === 1}>
            {step < 2 ? demonSlayer.title : typedTitle}
          </span>
          {#if step === 2}<span class="demo__caret"></span>{/if}
        </div>
        {#if step < 3}
          {@render exampleRow(onigiri, 'neutral', { score: '100%' })}
        {/if}
        <div data-demo-target="candidate">
          {@render exampleRow(kimetsu, finished ? 'info' : 'neutral', {
            selected: finished,
            score: '100%'
          })}
        </div>
      {:else}
        <div data-demo-target="candidate">
          {@render exampleRow(mob, finished ? 'info' : 'neutral', {
            selected: finished,
            score: '94%',
            star: true,
            softWarning: scenario.id === 'reassign' && !finished
          })}
        </div>
      {/if}
    </div>
  </div>

  {#if scenario.id === 'ignore'}
    <div class="bg-base-300 col-span-full px-3 py-1.5">
      <span class="badge badge-warning badge-soft badge-sm font-semibold">
        Do sprawdzenia <strong>{finished ? 0 : 1}</strong>
      </span>
    </div>
  {/if}

  {#if scenario.id === 'reassign' && step === 1}
    <div
      class="demo__dialog absolute inset-0 z-10 grid place-items-center bg-black/55 p-3"
    >
      <div class="bg-base-100 rounded-box w-full max-w-sm p-4 shadow-xl">
        <div class="flex items-start gap-2">
          <span
            class="icon-[lucide--triangle-alert] text-warning mt-0.5 size-4 shrink-0"
          ></span>
          <div>
            <p class="text-sm font-bold">Wpis jest już używany</p>
            <p class="text-muted mt-1 text-xs">
              Wpis „{previousMob.title}” utraci to dopasowanie.
            </p>
          </div>
        </div>
        <div class="mt-3 flex flex-wrap justify-end gap-1.5">
          <span class="btn btn-ghost btn-xs">Zostaw bez zmian</span>
          <span class="btn btn-warning btn-xs" data-demo-target="confirm">
            Wybierz mimo to
          </span>
        </div>
      </div>
    </div>
  {/if}

  {#if scenario.id === 'ignore' && step === 1}
    <ul
      class="demo__menu menu bg-base-100 rounded-box border-base-content/10 z-10 border p-1 shadow-xl"
      bind:this={contextMenu}
      style:--menu-x={menuPosition.x + 'px'}
      style:--menu-y={menuPosition.y + 'px'}
    >
      <li>
        <span>
          <span class="icon-[lucide--copy] size-4"></span>
          <span class="truncate">Kopiuj tytuł</span>
        </span>
      </li>
      <li>
        <span>
          <span class="icon-[lucide--external-link] size-4"></span>
          <span class="truncate">Otwórz stronę źródłową</span>
        </span>
      </li>
      <li class="border-base-content/10 mt-1 border-t pt-1 opacity-40">
        <span>
          <span class="icon-[lucide--rotate-ccw] size-4"></span>
          <span class="truncate">Resetuj wpis</span>
        </span>
      </li>
      <li>
        <span
          data-demo-target="ignore"
          class={menuItemHovered ? 'bg-base-content/10' : ''}
        >
          <span class="icon-[lucide--eye-off] size-4"></span>
          <span class="truncate">Ignoruj wpis</span>
        </span>
      </li>
    </ul>
  {/if}

  <span class="demo__cursor icon-[lucide--mouse-pointer-2]" bind:this={pointer}
  ></span>
</div>

{#snippet exampleRow(
  anime: ExampleAnime,
  tone: EntryRowTone,
  options: RowOptions = {}
)}
  <EntryRow
    {tone}
    isSelected={options.selected ?? false}
    ariaLabel={anime.title}
    title={anime.title}
    showIndicator={!options.score || options.star === true}
    indicator={options.star ? 'star' : 'bar'}
    rounded={options.score !== undefined}
    softWarning={options.softWarning ?? false}
    compact
    separated={false}
    onSelect={() => {}}
  >
    <p class="truncate text-sm font-semibold">{anime.title}</p>
    <RowMetadataBadges
      items={[
        { label: anime.year, tone: 'year' },
        { label: 'Serial TV', tone: 'anime-type' }
      ]}
    />
    {#snippet meta()}
      <span class="text-xs font-semibold">
        {options.score ?? options.status ?? 'Obejrzane'}
      </span>
      <span class="text-muted text-xs">
        {options.score
          ? `${anime.episodes} odc.`
          : `${anime.episodes} / ${anime.episodes} odc.`}
      </span>
    {/snippet}
  </EntryRow>
{/snippet}

<style>
  .demo__toolbar {
    display: grid;
    grid-template-columns: minmax(4rem, 1fr) minmax(0, max-content) minmax(
        4rem,
        1fr
      );
    align-items: center;
    gap: 0.25rem;
  }

  .demo__scenario-list {
    position: relative;
    isolation: isolate;
    grid-column: 2;
  }

  .demo__scenario-list button {
    position: relative;
    z-index: 1;
  }

  .demo__scenario-list button[aria-pressed='true'] {
    border-color: transparent;
    background: transparent;
    color: var(--ctp-mocha-crust);
    box-shadow: none;
  }

  .demo__highlight {
    position: absolute;
    z-index: 0;
    display: block;
    top: 0;
    left: 0;
    border-radius: var(--radius-field);
    background: var(--ctp-mocha-mauve);
    pointer-events: none;
    transition:
      transform 230ms ease-out,
      width 230ms ease-out,
      height 230ms ease-out;
  }

  .demo__controls {
    grid-column: 3;
    justify-self: end;
  }

  .demo__description {
    display: grid;
  }

  .demo__description > span:not(.sr-only) {
    grid-area: 1 / 1;
  }

  .demo {
    position: relative;
    display: grid;
    block-size: 13.5rem;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    grid-template-rows: minmax(0, 1fr);
    gap: 1px;
    overflow: hidden;
    border: 1px solid
      color-mix(in oklab, var(--color-base-content) 12%, transparent);
    border-radius: var(--radius-box);
    background: color-mix(
      in oklab,
      var(--color-base-content) 12%,
      var(--color-base-300)
    );
  }

  .demo__pane {
    min-width: 0;
    background: var(--color-base-200);
  }

  .demo__label {
    background: var(--color-base-300);
    padding: 0.45rem 0.75rem;
    color: var(--color-primary);
    font-size: 0.75rem;
    font-weight: 650;
  }

  .demo__rows {
    display: grid;
    min-width: 0;
    gap: 0.25rem;
    padding: 0.5rem;
  }

  .demo__rows :global(.entry-row--rounded) {
    width: 100%;
    margin: 0;
  }

  .demo__input {
    gap: 0;
  }

  .demo__input--focused {
    border-color: var(--color-info);
  }

  .demo__selection {
    background: var(--color-info);
    color: var(--color-info-content);
  }

  .demo__caret {
    width: 1px;
    height: 1em;
    background: currentColor;
  }

  .demo__dialog {
    animation: show-illustration 160ms ease-out;
  }

  .demo__menu {
    position: absolute;
    top: var(--menu-y);
    left: max(4px, min(var(--menu-x), calc(100% - 14.25rem)));
    width: min(14rem, calc(100% - 8px));
    transform-origin: top left;
    animation: show-illustration 100ms ease-out;
  }

  .demo__menu li > span {
    grid-template-columns: 1rem minmax(0, 1fr);
    gap: calc(var(--spacing) * 2);
  }

  .demo__cursor {
    position: absolute;
    z-index: 11;
    top: 0;
    left: 0;
    width: 1.25rem;
    height: 1.25rem;
    color: white;
    filter: drop-shadow(0 1px 2px var(--color-base-300));
    opacity: 0;
    pointer-events: none;
    transform-origin: top left;
    transition: opacity 150ms ease;
  }

  @media (width < 38rem) {
    .demo {
      block-size: 20.25rem;
      grid-template-columns: minmax(0, 1fr);
      grid-template-rows: auto minmax(0, 1fr);
    }
  }

  @media (width < 27rem) {
    .demo__toolbar {
      grid-template-columns: minmax(0, 1fr);
    }

    .demo__scenario-list,
    .demo__controls {
      grid-column: 1;
    }

    .demo__controls {
      justify-self: center;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .demo__highlight {
      transition: none;
    }

    .demo__cursor {
      display: none;
    }
  }

  @keyframes show-illustration {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
</style>
