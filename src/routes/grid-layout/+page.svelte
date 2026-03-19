<script lang="ts">
  import { backend } from "$services/backend";

  let status = $state("");

  async function applyLayout() {
    await backend.gridApplyLayout();
    status = "Grid layout apply command sent";
  }
</script>

<section class="card">
  <h2>Grid Layout</h2>
  <p>Bulk layout application endpoint is wired; preview algorithm UI is next.</p>
  <button onclick={applyLayout}>Apply layout (stub)</button>
  {#if status}<p>{status}</p>{/if}
</section>
