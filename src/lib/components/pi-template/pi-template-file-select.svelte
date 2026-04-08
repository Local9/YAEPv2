<script lang="ts">
  let {
    templateFiles,
    selectedFile = $bindable(""),
    onfilechange
  }: {
    templateFiles: string[];
    selectedFile: string;
    onfilechange?: () => void;
  } = $props();

  let filterText = $state("");
  let highlightIndex = $state(-1);
  /** Panel visible while the control is active (focus inside or interacting). */
  let panelOpen = $state(false);
  let rootEl: HTMLDivElement | null = null;

  const listboxId = "pi-template-file-listbox";

  /** File or path segment without the last extension (e.g. `a/b.json` -> `a/b`). */
  function fileNameWithoutExtension(pathOrName: string): string {
    const base = pathOrName.replace(/^.*[/\\]/, "");
    const i = base.lastIndexOf(".");
    if (i <= 0) return base;
    return base.slice(0, i);
  }

  /** Empty pattern lists all files. `*` is a wildcard; without `*`, match is substring (case-insensitive). */
  function matchFilter(pattern: string, fileName: string): boolean {
    const p = pattern.trim();
    if (!p) return true;
    if (p.includes("*")) {
      const parts = p.split("*").map((s) => s.replace(/[.+?^${}()|[\]\\]/g, "\\$&"));
      return new RegExp(`^${parts.join(".*")}$`, "i").test(fileName);
    }
    return fileName.toLowerCase().includes(p.toLowerCase());
  }

  let filteredFiles = $derived(
    templateFiles.filter(
      (f) =>
        matchFilter(filterText, f) || matchFilter(filterText, fileNameWithoutExtension(f))
    )
  );

  $effect(() => {
    filterText = fileNameWithoutExtension(selectedFile);
  });

  $effect(() => {
    filterText;
    highlightIndex = -1;
  });

  function openPanel() {
    if (templateFiles.length === 0) return;
    panelOpen = true;
  }

  function closePanel() {
    panelOpen = false;
    highlightIndex = -1;
  }

  function onRootFocusOut(e: FocusEvent) {
    const next = e.relatedTarget as Node | null;
    if (next && rootEl?.contains(next)) return;
    closePanel();
  }

  function onInputFocus() {
    openPanel();
  }

  function onInputInput() {
    openPanel();
  }

  function selectFile(fileName: string) {
    selectedFile = fileName;
    filterText = fileNameWithoutExtension(fileName);
    highlightIndex = -1;
    closePanel();
    onfilechange?.();
  }

  function onInputKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      closePanel();
      return;
    }

    const list = filteredFiles;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (!panelOpen) openPanel();
      if (list.length === 0) return;
      highlightIndex =
        highlightIndex < 0 ? 0 : Math.min(highlightIndex + 1, list.length - 1);
      return;
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      if (!panelOpen) openPanel();
      if (list.length === 0) return;
      highlightIndex =
        highlightIndex <= 0 ? list.length - 1 : highlightIndex - 1;
      return;
    }

    if (!panelOpen) return;

    if (e.key === "Enter") {
      e.preventDefault();
      const pick =
        highlightIndex >= 0 ? list[highlightIndex] : list.length === 1 ? list[0] : undefined;
      if (pick) selectFile(pick);
      return;
    }
  }
</script>

<div class="max-w-xl space-y-2">
  <label for="pi-template-file-filter" class="text-sm font-medium">Template file</label>
  <div class="relative" bind:this={rootEl} onfocusout={onRootFocusOut}>
    <input
      id="pi-template-file-filter"
      type="text"
      role="combobox"
      aria-autocomplete="list"
      aria-controls={listboxId}
      aria-expanded={panelOpen}
      aria-haspopup="listbox"
      autocomplete="off"
      spellcheck="false"
      placeholder="Filter; use * as wildcard (leave empty for all)"
      class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm outline-none ring-offset-background focus-visible:ring-2 focus-visible:ring-ring"
      bind:value={filterText}
      onfocus={onInputFocus}
      oninput={onInputInput}
      onkeydown={onInputKeydown}
    />

    <ul
      id={listboxId}
      role="listbox"
      aria-label="Matching template files"
      aria-hidden={!panelOpen}
      class:hidden={!panelOpen}
      class="absolute left-0 right-0 top-full z-50 mt-1 max-h-52 overflow-y-auto rounded-md border border-border bg-popover py-1 text-popover-foreground shadow-md"
    >
      {#if filteredFiles.length === 0}
        <li class="px-3 py-2 text-muted-foreground" role="presentation">No matching files.</li>
      {:else}
        {#each filteredFiles as fileName, i (fileName)}
          <li role="presentation">
            <button
              type="button"
              tabindex="-1"
              role="option"
              aria-selected={fileName === selectedFile}
              class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm hover:bg-muted/80 focus-visible:bg-muted/80 focus-visible:outline-none {i ===
              highlightIndex
                ? 'bg-muted'
                : ''} {fileName === selectedFile ? 'font-medium text-foreground' : 'text-foreground'}"
              onmousedown={(e) => e.preventDefault()}
              onclick={() => selectFile(fileName)}
              onmouseenter={() => (highlightIndex = i)}
            >
              {fileNameWithoutExtension(fileName)}
            </button>
          </li>
        {/each}
      {/if}
    </ul>
  </div>
</div>
