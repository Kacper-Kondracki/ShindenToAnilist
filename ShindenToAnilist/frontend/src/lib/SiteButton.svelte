<script>
    export let image;
    export let text;
    export let selected = false;
</script>

<button
    on:click
    class:btn-accent={selected}
    style="max-width: 290px;"
    class="btn btn-square flex-shrink size-full btn-lg p-1"
>
    <div class="relative">
        <img src={image} class="w-full rounded-lg" alt="" />
        <div
            class="absolute glass bottom-0 px-4 py-3 text-white bg-black/60 w-full rounded-lg"
        >
            {text}
        </div>
    </div>
</button>
