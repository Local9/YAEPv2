<script lang="ts">
  import "../app.css";

  const sections = [
    { href: "/", label: "Dashboard" },
    { href: "/profiles", label: "Profiles" },
    { href: "/thumbnail-settings", label: "Thumbnail Settings" },
    { href: "/client-grouping", label: "Client Grouping" },
    { href: "/grid-layout", label: "Grid Layout" },
    { href: "/process-management", label: "Process Management" },
    { href: "/mumble-links", label: "Mumble Links" },
    { href: "/eve-profiles", label: "EVE Profiles" },
    { href: "/settings", label: "Settings" }
  ];
</script>

<div class="shell">
  <aside class="sidebar">
    <h1>YAEP Rust</h1>
    <nav>
      {#each sections as section (section.href)}
        <a href={section.href}>{section.label}</a>
      {/each}
    </nav>
  </aside>
  <main class="content">
    <slot />
  </main>
</div>
