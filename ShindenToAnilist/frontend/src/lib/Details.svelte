<script>
  import { onMount } from "svelte";
  import SearchIcon from "virtual:icons/mdi/Search";
  import { Export, Search } from "../../wailsjs/go/main/App.js";
  import { BrowserOpenURL } from "../../wailsjs/runtime/runtime.js";

  export let anime;
  let selectedAnime = null;
  let searchPromise = null;
  let selectedSearchAnime = null;

  function SelectSearchAnime(anime) {
    selectedSearchAnime = anime;
    if (compare(selectedAnime.search_anime.fix, anime)) {
      selectedSearchAnime = null;
      selectedAnime.search_anime.fix = undefined;
      selectedAnime.search_anime.isFixed = false;
      selectedAnime.search_anime = selectedAnime.search_anime;
    } else {
      selectedAnime.search_anime.fix = selectedSearchAnime;
      selectedAnime.search_anime.isFixed = true;
      selectedAnime.search_anime = selectedAnime.search_anime;
    }
    searchPromise = searchPromise;
  }

  let animeSearchText = "";

  function SearchAnime() {
    if (selectedAnime?.search_anime?.fix != null) {
      selectedSearchAnime = selectedAnime.search_anime.fix;
    }
    searchPromise = Search(animeSearchText);
  }

  function FilterMultiple(list) {
    if (selectedAnime.db_anime != null) {
      return list.filter(x => !selectedAnime.db_anime.some(y => JSON.stringify(y) == JSON.stringify(x)));
    }
    return list;
  }

  function ContainsFix(list) {
    if (selectedAnime?.search_anime?.fix == null) {
      return true;
    }
    // if (selectedAnime?.db_anime != null && selectedAnime.db_anime.includes(selectedAnime.search_anime.fix)) {
    //   return true;
    // }

    if (selectedAnime?.db_anime != null && selectedAnime.db_anime.some(x => compare(x, selectedAnime.search_anime.fix))) {
      return true;
    }

    return list.some(x => compare(x, selectedAnime.search_anime.fix));
  }

  function compare(a, b) {
    return JSON.stringify(a) == JSON.stringify(b);
  }

  async function DoExport() {
    let animes = await anime;

    let all = [];
    animes.multipleAnime.forEach(x => {
      all.push(x.search_anime?.fix);
    });
    animes.failAnime.forEach(x => {
      all.push(x.search_anime?.fix);
    });


    let res = await Export(JSON.stringify(all));
  }

</script>

<div class="min-h-0 h-full w-full overflow-hidden">
  <div
    class="grid grid-cols-2 grid-rows-1 h-full ">
    <div class="col-span-2 row-start-3 row-end-3 flex items-center">
      <button on:click={DoExport} class="btn btn-accent w-full">Eksportuj</button>
    </div>
    <div class="col-start-2 col-end-3 row-start-1 row-end-2 grid grid-rows-3">
      <div class="p-4 row-span-2 flex flex-col">
        <div class="mb-4">
          <label class="input input-bordered input-accent !outline-none flex items-center gap-2">
            <input on:change={SearchAnime} bind:value="{animeSearchText}" type="text" class="grow"
                   placeholder="Szukaj anime" />
            <SearchIcon></SearchIcon>
          </label>
          {#if selectedAnime?.search_anime?.source != null}
            <!--{}-->
            <button on:click={() => BrowserOpenURL(selectedAnime.search_anime.source)} class="text-sm text-blue-500 mt-2 cursor-pointer text-left">
              {selectedAnime.search_anime.source}
            </button>
          {/if}
        </div>
        <div class="min-h-0">
          {#if searchPromise != null}
            {#await searchPromise}
              <p></p>
            {:then result}
              <div
                class="overflow-auto scrollbar-thumb-rounded-full scrollbar-track-rounded-full scrollbar scrollbar-thumb-accent scrollbar-track-base-200 max-h-full">
                <table class="table">
                  <thead>
                  <tr>
                  </tr>
                  </thead>
                  <tbody>
                  {#if !ContainsFix(result)}
                    <tr
                      class="block rounded-xl
                     {compare(selectedAnime.search_anime.fix, selectedSearchAnime) ? 'bg-base-300' : 'hover'}"
                      on:click={() => {SelectSearchAnime(selectedAnime.search_anime.fix)}}>
                      <th
                        class="">{selectedAnime.search_anime.fix.title}
                      </th>
                    </tr>
                  {/if}
                  {#if selectedAnime.db_anime != null}
                    {#each selectedAnime.db_anime as anime}
                      <tr
                        class="block rounded-xl
                     {compare(anime, selectedSearchAnime) ? 'bg-base-300' : 'hover'}"
                        on:click={() => {SelectSearchAnime(anime)}}>
                        <th
                          class="">⭐ {anime.title}</th>
                      </tr>
                    {/each}
                  {/if}
                  {#each FilterMultiple(result) as anime}
                    <tr
                      class="block rounded-xl
                     {compare(anime, selectedSearchAnime) ? 'bg-base-300' : 'hover'}"
                      on:click={() => {SelectSearchAnime(anime)}}>
                      <th
                        class="">{anime.title}</th>
                    </tr>
                  {/each}
                  </tbody>
                </table>
              </div>
            {/await}
          {/if}
        </div>
      </div>
      <div class="p-4 row-start-3">
        {#if selectedSearchAnime !== null}
          <div class="artboard-demo size-full grid grid-cols-3">
            <div class="size-full overflow-hidden rounded-2xl">
              <img class="object-cover size-full"
                   src="{selectedSearchAnime.picture}" alt="Brak obrazka">
            </div>
            <div class="size-full col-span-2 p-2 flex flex-col">
              <p class="font-bold text-accent text-center break-words line-clamp-3"
                 style="font-size: 2.2vh;">{selectedSearchAnime.title}</p>
              <div class="flex flex-col justify-end flex-grow">
                <p><span class="text-info font-bold">Rok:</span> {selectedSearchAnime.animeSeason.year}</p>
                <p><span class="text-info font-bold">Typ:</span> {selectedSearchAnime.type}</p>
                <p><span class="text-info font-bold">Odcinki:</span> {selectedSearchAnime.episodes}</p>
              </div>
            </div>
          </div>
        {/if}
      </div>
    </div>
    <div
      class="overflow-auto scrollbar-thumb-rounded-full scrollbar-track-rounded-full scrollbar scrollbar-thumb-accent scrollbar-track-base-200 max-h-full">
      <table class="table">
        <thead>
        <tr>
        </tr>
        </thead>
        <tbody>
        {#if anime != null}
          {#await anime}
          {:then result}
            {#each result.multipleAnime as anime, index}
              <tr
                class="block
                     {anime.search_anime === selectedAnime?.search_anime ? 'bg-base-300' : 'hover'}
                     {anime.search_anime.isFixed ? 'border-l-8 border-l-success' : 'border-l-8 border-l-warning'}"
                on:click={() => {selectedAnime = anime; animeSearchText = selectedAnime.search_anime.title; SearchAnime()}}>
                <th
                  class="">{anime.search_anime.title}</th>
              </tr>
            {/each}
            {#each result.failAnime as anime, index}
              <tr
                class="block
                     {anime.search_anime === selectedAnime?.search_anime ? 'bg-base-300' : 'hover'}
                     {anime.search_anime.isFixed ? 'border-l-8 border-l-success' : 'border-l-8 border-l-error'}"
                on:click={() => {selectedAnime = anime; animeSearchText = selectedAnime.search_anime.title; SearchAnime()}}>
                <th
                  class="">{anime.search_anime.title}</th>
              </tr>
            {/each}

          {:catch error}
            <p>error</p>
          {/await}
        {/if}

        </tbody>
      </table>
    </div>
  </div>
</div>