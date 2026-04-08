import type { Edge, Node } from "@xyflow/svelte";
import type { PiTemplate } from "$models/domain";

export type PiPlacementNodeData = {
  typeId: number;
  structureId: number | null;
  index: number;
};

const FLOW_WIDTH = 880;
const FLOW_HEIGHT = 560;
const PADDING = 72;

function nodeIdForPlacementIndex(index: number): string {
  return `p-${index}`;
}

/** True when every segment reference is a 1-based row index into `P` (common in exports). */
function pathUsesOneBasedPlacementIndices(path: number[], placementCount: number): boolean {
  if (placementCount === 0) return false;
  for (const v of path) {
    if (!Number.isFinite(v) || Math.floor(v) !== v || v < 1 || v > placementCount) {
      return false;
    }
  }
  return true;
}

/**
 * First placement row whose `S` matches (for templates where `R[].P` uses structure IDs).
 */
function structureIdToFirstNodeId(pins: PiTemplate["P"]): Map<number, string> {
  const map = new Map<number, string>();
  pins.forEach((pin, index) => {
    if (pin.S != null && !map.has(pin.S)) {
      map.set(pin.S, nodeIdForPlacementIndex(index));
    }
  });
  return map;
}

/**
 * Maps PI template placements to flow node positions using normalized La/Lo from the file.
 * One node per `P[]` row (`p-0` … `p-(n-1)`), so node count always equals `P.length`.
 *
 * `R[].P` is interpreted as **1-based placement indices** when every value lies in `[1, P.length]`;
 * otherwise it is treated as **structure IDs** (`S`) with first matching placement winning.
 */
export function piTemplateToFlowElements(template: PiTemplate): {
  nodes: Node<PiPlacementNodeData>[];
  edges: Edge[];
} {
  const pins = template.P;
  if (pins.length === 0) {
    return { nodes: [], edges: [] };
  }

  let minLa = Infinity;
  let maxLa = -Infinity;
  let minLo = Infinity;
  let maxLo = -Infinity;
  for (const pin of pins) {
    minLa = Math.min(minLa, pin.La);
    maxLa = Math.max(maxLa, pin.La);
    minLo = Math.min(minLo, pin.Lo);
    maxLo = Math.max(maxLo, pin.Lo);
  }
  const spanLa = maxLa - minLa || 1;
  const spanLo = maxLo - minLo || 1;
  const innerW = FLOW_WIDTH - 2 * PADDING;
  const innerH = FLOW_HEIGHT - 2 * PADDING;

  const nodes: Node<PiPlacementNodeData>[] = pins.map((pin, index) => {
    const x = PADDING + ((pin.Lo - minLo) / spanLo) * innerW;
    const y = PADDING + ((pin.La - minLa) / spanLa) * innerH;
    return {
      id: nodeIdForPlacementIndex(index),
      type: "piPlacement",
      position: { x, y },
      data: {
        typeId: pin.T,
        structureId: pin.S,
        index
      }
    };
  });

  const byStructureId = structureIdToFirstNodeId(pins);

  const edges: Edge[] = [];
  template.R.forEach((route, routeIndex) => {
    const path = route.P;
    if (path.length < 2) return;
    const usePlacementIndex = pathUsesOneBasedPlacementIndices(path, pins.length);
    for (let seg = 0; seg < path.length - 1; seg++) {
      const a = path[seg];
      const b = path[seg + 1];
      let source: string | undefined;
      let target: string | undefined;
      if (usePlacementIndex) {
        source = nodeIdForPlacementIndex(a - 1);
        target = nodeIdForPlacementIndex(b - 1);
      } else {
        source = byStructureId.get(a);
        target = byStructureId.get(b);
      }
      if (!source || !target) continue;
      edges.push({
        id: `r-${routeIndex}-${seg}-${source}-${target}`,
        source,
        target,
        label: `Q ${route.Q} / T ${route.T}`
      });
    }
  });

  return { nodes, edges };
}
