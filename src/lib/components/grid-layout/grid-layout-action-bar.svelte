<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import LayoutGridIcon from "@lucide/svelte/icons/layout-grid";
  import PlayIcon from "@lucide/svelte/icons/play";
  import DownloadIcon from "@lucide/svelte/icons/download";

  interface Props {
    activeProfileId: number | null;
    exportBusy: boolean;
    onGeneratePreview: () => void;
    onApplyLayout: () => void;
    onExportPrefs: () => void;
  }

  let {
    activeProfileId,
    exportBusy,
    onGeneratePreview,
    onApplyLayout,
    onExportPrefs,
  }: Props = $props();
</script>

<div class="mt-4 flex flex-wrap gap-2">
  <Button onclick={onGeneratePreview} class="gap-2">
    <LayoutGridIcon class="size-4 shrink-0" aria-hidden="true" />
    Generate Preview
  </Button>
  <Button onclick={onApplyLayout} variant="secondary" class="gap-2">
    <PlayIcon class="size-4 shrink-0" aria-hidden="true" />
    Apply Layout
  </Button>
  <Button
    onclick={() => void onExportPrefs()}
    variant="outline"
    class="gap-2"
    disabled={activeProfileId == null || exportBusy}
  >
    <DownloadIcon class="size-4 shrink-0" aria-hidden="true" />
    Export settings
  </Button>
</div>
