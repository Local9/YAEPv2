<script lang="ts">
  import type { GridLayoutPreviewItem } from "$models/domain";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from "$lib/components/ui/table";

  interface Props {
    preview: GridLayoutPreviewItem[];
  }

  let { preview }: Props = $props();
</script>

<div class="mt-6 overflow-x-auto">
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead>Window Title</TableHead>
        <TableHead>X</TableHead>
        <TableHead>Y</TableHead>
        <TableHead>Width</TableHead>
        <TableHead>Height</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      {#each preview as item (item.windowTitle)}
        <TableRow>
          <TableCell>{item.windowTitle}</TableCell>
          <TableCell>{item.x}</TableCell>
          <TableCell>{item.y}</TableCell>
          <TableCell>{item.width}</TableCell>
          <TableCell>{item.height}</TableCell>
        </TableRow>
      {/each}
    </TableBody>
  </Table>
</div>
