<script lang="ts">
  import type { SelectedPlacementDetail } from "$lib/pi-template/pi-template-page.types";
  import { formatIsk } from "$lib/pi-template/isk-format";

  const numFormat = new Intl.NumberFormat("en-US", { maximumFractionDigits: 2 });

  let {
    selection,
    placementCount
  }: {
    selection: SelectedPlacementDetail | null;
    placementCount: number;
  } = $props();
</script>

<aside
  class="w-full shrink-0 rounded-md border bg-card p-3 text-sm shadow-sm lg:w-80 lg:max-w-[min(100%,20rem)]"
  aria-label="Selected structure details"
>
  {#if selection}
    <h3 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Selected structure</h3>
    <p class="mt-2 text-base font-semibold leading-snug text-foreground">
      {selection.typeDetails?.name?.trim() ?? `Type ${selection.typeId}`}
    </p>
    {#if selection.typeDetails?.groupName?.trim()}
      <p class="mt-1 text-xs text-muted-foreground">{selection.typeDetails.groupName.trim()}</p>
    {/if}

    {#if selection.typeDetails?.basePrice != null}
      <div class="mt-3 rounded-md border border-border/60 bg-muted/30 px-3 py-2">
        <p class="text-xs font-medium text-muted-foreground">SDE base price</p>
        <p class="mt-0.5 text-lg font-semibold tabular-nums text-foreground">
          {formatIsk(selection.typeDetails.basePrice)}
        </p>
      </div>
    {:else}
      <p class="mt-3 text-xs text-muted-foreground">No base price listed in SDE for this type.</p>
    {/if}

    <dl class="mt-3 space-y-2 border-t border-border pt-3 text-xs">
      <div class="flex justify-between gap-2">
        <dt class="text-muted-foreground">Type ID</dt>
        <dd class="font-mono tabular-nums">{selection.typeId}</dd>
      </div>
      <div class="flex justify-between gap-2">
        <dt class="text-muted-foreground">Structure ID (S)</dt>
        <dd class="font-mono tabular-nums">{selection.structureId ?? "null"}</dd>
      </div>
      <div class="flex justify-between gap-2">
        <dt class="text-muted-foreground">Placement row</dt>
        <dd class="font-mono tabular-nums">{selection.index + 1} of {placementCount}</dd>
      </div>
      <div class="flex justify-between gap-2">
        <dt class="text-muted-foreground">H (layer)</dt>
        <dd class="font-mono tabular-nums">{selection.layerH}</dd>
      </div>
      {#if selection.typeDetails?.volume != null}
        <div class="flex justify-between gap-2">
          <dt class="text-muted-foreground">Volume</dt>
          <dd class="font-mono tabular-nums">
            {numFormat.format(selection.typeDetails.volume)} m3
          </dd>
        </div>
      {/if}
      {#if selection.typeDetails?.mass != null}
        <div class="flex justify-between gap-2">
          <dt class="text-muted-foreground">Mass</dt>
          <dd class="font-mono tabular-nums">
            {numFormat.format(selection.typeDetails.mass)} kg
          </dd>
        </div>
      {/if}
      {#if selection.typeDetails?.portionSize != null}
        <div class="flex justify-between gap-2">
          <dt class="text-muted-foreground">Portion size</dt>
          <dd class="font-mono tabular-nums">{selection.typeDetails.portionSize}</dd>
        </div>
      {/if}
      {#if selection.typeDetails?.published != null}
        <div class="flex justify-between gap-2">
          <dt class="text-muted-foreground">Published</dt>
          <dd>{selection.typeDetails.published ? "Yes" : "No"}</dd>
        </div>
      {/if}
    </dl>

    <div class="mt-3 space-y-3 border-t border-border pt-3 text-xs">
      {#if selection.incomingRoutes.length > 0}
        <div>
          <h4 class="font-semibold uppercase tracking-wide text-muted-foreground">Incoming routes</h4>
          <ul class="mt-1.5 space-y-1">
            {#each selection.incomingRoutes as r, i (i)}
              <li class="flex flex-wrap items-baseline gap-x-1.5 gap-y-0">
                <span class="font-mono tabular-nums text-muted-foreground">Q{r.quantity}</span>
                <span class="text-foreground">{r.materialLabel}</span>
                <span class="font-mono text-[10px] text-muted-foreground">T{r.routeTypeId}</span>
              </li>
            {/each}
          </ul>
        </div>
      {/if}
      {#if selection.outgoingRoutes.length > 0}
        <div>
          <h4 class="font-semibold uppercase tracking-wide text-muted-foreground">Outgoing routes</h4>
          <ul class="mt-1.5 space-y-1">
            {#each selection.outgoingRoutes as r, i (i)}
              <li class="flex flex-wrap items-baseline gap-x-1.5 gap-y-0">
                <span class="font-mono tabular-nums text-muted-foreground">Q{r.quantity}</span>
                <span class="text-foreground">{r.materialLabel}</span>
                <span class="font-mono text-[10px] text-muted-foreground">T{r.routeTypeId}</span>
              </li>
            {/each}
          </ul>
        </div>
      {/if}
      {#if selection.incomingRoutes.length === 0 && selection.outgoingRoutes.length === 0}
        <p class="text-muted-foreground">No material routes through this placement.</p>
      {/if}
    </div>
  {:else}
    <p class="text-sm text-muted-foreground">
      Select a structure on the map to see template fields and SDE type details.
    </p>
  {/if}
</aside>
