<script lang="ts">
  import AnimatedGridPanel from "./AnimatedGridPanel.svelte";

  let { providerLabel }: { providerLabel: string } = $props();
</script>

<section class="app-content">
  <AnimatedGridPanel class="grid place-items-center overflow-hidden surface-panel">
    <div class="isolate grid max-w-3xl justify-items-center gap-2 px-6 text-center">
      <p class="text-2xl font-bold md:text-4xl">
        Wczytaj listę, żeby rozpocząć dopasowywanie
      </p>
      <p class="text-base font-medium text-muted md:text-xl">
        Aktywny import z {providerLabel}, pozostałe źródła w budowie
      </p>
    </div>
  </AnimatedGridPanel>
</section>
