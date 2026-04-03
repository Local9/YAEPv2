<script lang="ts">
  import HouseIcon from "@lucide/svelte/icons/house";
  import type { BrowserQuickLink, WidgetBrowserFrame } from "$models/domain";
  import WidgetWrapper from "$lib/components/widget-overlay/widget-wrapper.svelte";

  let {
    browser = $bindable(),
    pinned = $bindable(false),
    quickLinks,
    rootEl = $bindable<HTMLElement | undefined>(undefined),
    onPersist,
    onPinnedPersist
  }: {
    browser: WidgetBrowserFrame;
    pinned?: boolean;
    quickLinks: BrowserQuickLink[];
    rootEl?: HTMLElement | undefined;
    onPersist: () => void | Promise<void>;
    /** Persisted when the pin button is toggled (e.g. full overlay settings). */
    onPinnedPersist: () => void | Promise<void>;
  } = $props();

  let urlDraft = $state(browser.url);

  const hasPage = $derived(browser.url.trim().length > 0);

  $effect(() => {
    urlDraft = browser.url;
  });

  function normalizeUrl(raw: string): string {
    const t = raw.trim();
    if (!t) return "";
    if (/^https?:\/\//i.test(t)) return t;
    return `https://${t}`;
  }

  function onNavigate() {
    const next = normalizeUrl(urlDraft);
    urlDraft = next;
    browser = { ...browser, url: next };
    void onPersist();
  }

  function openQuickLink(link: BrowserQuickLink) {
    const next = link.url.trim();
    urlDraft = next;
    browser = { ...browser, url: next };
    void onPersist();
  }

  function goHome() {
    urlDraft = "";
    browser = { ...browser, url: "" };
    void onPersist();
  }

  function onUrlKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      onNavigate();
    }
  }
</script>

<WidgetWrapper
  title="Browser"
  shellAriaLabel="Embedded browser widget"
  bind:frame={browser}
  bind:pinned
  bind:rootEl
  onPersist={onPersist}
  onPinnedPersist={onPinnedPersist}
>
  {#snippet toolbar()}
    <div
      class="browser-toolbar flex shrink-0 items-center gap-1.5 bg-muted text-foreground border-b border-border px-[10px] py-2"
      role="group"
      aria-label="Address and navigation"
    >
      {#if hasPage}
        <button
          type="button"
          class="browser-home flex shrink-0 items-center justify-center p-[4px] rounded border border-border bg-background text-foreground cursor-pointer hover:bg-accent hover:text-accent-foreground"
          title="Home (shortcuts)"
          aria-label="Home, show shortcuts"
          onclick={goHome}
        >
          <HouseIcon class="browser-home-icon h-4 w-4" aria-hidden="true" />
        </button>
      {/if}
      <input
        class="browser-url-input min-w-0 flex-1 rounded border border-border bg-background px-2 py-[6px] text-[12px] font-[ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace] text-foreground outline-none focus:border-ring focus:outline-none focus:shadow-[0_0_0_2px_color-mix(in_oklch,var(--ring)_35%,transparent)]"
        type="text"
        spellcheck="false"
        autocomplete="off"
        aria-label="Address"
        placeholder={hasPage ? "" : "Search or enter URL"}
        bind:value={urlDraft}
        onkeydown={onUrlKeydown}
      />
      <button
        type="button"
        class="browser-go shrink-0 rounded border border-border bg-secondary px-3 py-[5px] text-[12px] font-medium text-secondary-foreground cursor-pointer hover:bg-accent hover:text-accent-foreground"
        onclick={onNavigate}
      >
        Go
      </button>
    </div>
  {/snippet}

  {#snippet children()}
    {#if hasPage}
      <iframe
        class="browser-frame block flex-1 min-h-0 w-full border-0 bg-background"
        title="Browser widget content"
        src={browser.url}
        referrerpolicy="no-referrer-when-downgrade"
      ></iframe>
    {:else}
      <div class="browser-start flex-1 min-h-0 overflow-auto bg-background pt-4 pb-3 px-[14px]" aria-label="Shortcut links">
        <p class="browser-start-lead m-0 mb-3 text-[13px] font-semibold text-foreground">Open a page</p>
        <div class="browser-quick-links grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-2.5">
          {#each quickLinks as link (link.id)}
            <button
              type="button"
              class="browser-tile flex min-h-[72px] flex-col items-start justify-center gap-1 rounded-lg border border-border bg-card text-card-foreground text-left cursor-pointer p-[10px_12px] transition-[background-color,border-color] duration-120 ease-in-out hover:border-[color-mix(in_oklch,var(--ring)_50%,var(--border))] hover:[background:color-mix(in_oklch,var(--accent)_12%,var(--card))]"
              onclick={() => openQuickLink(link)}
            >
              <span class="browser-tile-title text-[13px] font-semibold leading-tight">{link.title}</span>
              <span class="browser-tile-url truncate text-[10px] leading-[1.3] text-muted-foreground">{link.url}</span>
            </button>
          {/each}
        </div>
      </div>
    {/if}
  {/snippet}
</WidgetWrapper>

<style>
  :global(html.dark) .browser-frame {
    color-scheme: dark;
  }

  :global(html:not(.dark)) .browser-frame {
    color-scheme: light;
  }
</style>
