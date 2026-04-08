<script lang="ts">
  let { text }: { text: string } = $props();
</script>

<div class="space-y-1 rounded-md border bg-muted/20 p-3">
  <h2 class="text-sm font-semibold">Comment</h2>
  <p class="text-sm wrap-break-word">{text || "No comment."}</p>
</div>
