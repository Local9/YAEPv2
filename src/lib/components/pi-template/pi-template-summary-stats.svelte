<script lang="ts">
  import type { PiTemplateSummaryStats } from "$lib/pi-template/pi-template-page.types";
  import { formatIsk } from "$lib/pi-template/isk-format";

  let { summary }: { summary: PiTemplateSummaryStats } = $props();
</script>

<dl
  class="grid grid-cols-4 gap-x-4 gap-y-2 rounded-md border bg-muted/10 px-3 py-2 text-xs"
>
  <div>
    <dt class="text-muted-foreground">Command Control Lv</dt>
    <dd class="font-mono tabular-nums">{summary.cmdCtrLv}</dd>
  </div>
  <div>
    <dt class="text-muted-foreground">Diameter</dt>
    <dd class="font-mono tabular-nums">{summary.diam}</dd>
  </div>
  <div>
    <dt class="text-muted-foreground">Planet type</dt>
    <dd class="min-w-0 font-mono text-[11px]">
      {#if summary.plnTypeName}
        <span class="wrap-break-word font-sans text-foreground">{summary.plnTypeName}</span>
        <span class="text-muted-foreground"> ({summary.plnId})</span>
      {:else}
        <span class="tabular-nums">{summary.plnId}</span>
      {/if}
    </dd>
  </div>
  <div>
    <dt class="text-muted-foreground">Total structure cost (SDE base)</dt>
    <dd class="font-mono tabular-nums text-sm text-foreground">
      {formatIsk(summary.totalStructureCostIsk)}
      {#if summary.placementsMissingBasePrice > 0}
        <span class="mt-0.5 block font-sans text-[10px] font-normal text-muted-foreground">
          {summary.placementsMissingBasePrice} placement(s) have no base price in SDE (excluded from sum).
        </span>
      {/if}
    </dd>
  </div>
  <div>
    <dt class="text-muted-foreground">Placements</dt>
    <dd class="font-mono tabular-nums">{summary.placements}</dd>
  </div>
  <div>
    <dt class="text-muted-foreground">Routes</dt>
    <dd class="font-mono tabular-nums">{summary.routes}</dd>
  </div>
  <div>
    <dt class="text-muted-foreground">Links (L)</dt>
    <dd class="font-mono tabular-nums">{summary.links}</dd>
  </div>
</dl>
