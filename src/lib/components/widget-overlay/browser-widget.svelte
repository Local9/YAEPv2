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
    <div class="browser-toolbar" role="group" aria-label="Address and navigation">
      {#if hasPage}
        <button
          type="button"
          class="browser-home"
          title="Home (shortcuts)"
          aria-label="Home, show shortcuts"
          onclick={goHome}
        >
          <HouseIcon class="browser-home-icon" aria-hidden="true" />
        </button>
      {/if}
      <input
        class="browser-url-input"
        type="text"
        spellcheck="false"
        autocomplete="off"
        aria-label="Address"
        placeholder={hasPage ? "" : "Search or enter URL"}
        bind:value={urlDraft}
        onkeydown={onUrlKeydown}
      />
      <button type="button" class="browser-go" onclick={onNavigate}>Go</button>
    </div>
  {/snippet}

  {#snippet children()}
    {#if hasPage}
      <iframe
        class="browser-frame"
        title="Browser widget content"
        src={browser.url}
        referrerpolicy="no-referrer-when-downgrade"
      ></iframe>
    {:else}
      <div class="browser-start" aria-label="Shortcut links">
        <p class="browser-start-lead">Open a page</p>
        <div class="browser-quick-links">
          {#each quickLinks as link (link.id)}
            <button type="button" class="browser-tile" onclick={() => openQuickLink(link)}>
              <span class="browser-tile-title">{link.title}</span>
              <span class="browser-tile-url">{link.url}</span>
            </button>
          {/each}
        </div>
      </div>
    {/if}
  {/snippet}
</WidgetWrapper>

<style>
  .browser-toolbar {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    gap: 6px;
    padding: 8px 10px;
    background: var(--muted);
    color: var(--foreground);
    border-bottom: 1px solid var(--border);
  }

  .browser-home {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
    padding: 4px;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: var(--background);
    color: var(--foreground);
    cursor: pointer;
  }

  .browser-home:hover {
    background: var(--accent);
    color: var(--accent-foreground);
  }

  .browser-home :global(.browser-home-icon) {
    width: 1rem;
    height: 1rem;
  }

  .browser-url-input {
    min-width: 0;
    flex: 1;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: var(--background);
    padding: 6px 8px;
    font-size: 12px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    color: var(--foreground);
    outline: none;
  }

  .browser-url-input:focus {
    border-color: var(--ring);
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--ring) 35%, transparent);
  }

  .browser-go {
    flex-shrink: 0;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: var(--secondary);
    padding: 5px 12px;
    font-size: 12px;
    font-weight: 500;
    color: var(--secondary-foreground);
    cursor: pointer;
  }

  .browser-go:hover {
    background: var(--accent);
    color: var(--accent-foreground);
  }

  .browser-frame {
    flex: 1;
    min-height: 0;
    width: 100%;
    border: 0;
    background: var(--background);
  }

  :global(html.dark) .browser-frame {
    color-scheme: dark;
  }

  :global(html:not(.dark)) .browser-frame {
    color-scheme: light;
  }

  .browser-start {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 16px 14px 12px;
    background: var(--background);
  }

  .browser-start-lead {
    margin: 0 0 12px;
    font-size: 13px;
    font-weight: 600;
    color: var(--foreground);
  }

  .browser-quick-links {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 10px;
  }

  .browser-tile {
    display: flex;
    min-height: 72px;
    flex-direction: column;
    align-items: flex-start;
    justify-content: center;
    gap: 4px;
    padding: 10px 12px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--card);
    color: var(--card-foreground);
    text-align: left;
    cursor: pointer;
    transition: background 0.12s ease, border-color 0.12s ease;
  }

  .browser-tile:hover {
    border-color: color-mix(in oklch, var(--ring) 50%, var(--border));
    background: color-mix(in oklch, var(--accent) 12%, var(--card));
  }

  .browser-tile-title {
    font-size: 13px;
    font-weight: 600;
    line-height: 1.25;
  }

  .browser-tile-url {
    max-width: 100%;
    font-size: 10px;
    line-height: 1.3;
    color: var(--muted-foreground);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
