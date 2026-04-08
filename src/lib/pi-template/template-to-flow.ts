import type { Edge, Node } from "@xyflow/svelte";
import { MarkerType } from "@xyflow/svelte";
import type { EveTypeSnapshot, PiTemplate } from "$models/domain";

/** Bounding box size (px) for circular placement nodes; must match `pi-placement-node`. */
export const PI_NODE_WIDTH = 96;
export const PI_NODE_HEIGHT = 96;

type CardinalSide = "top" | "right" | "bottom" | "left";

function nodeCenter(node: Node, w: number, h: number): { x: number; y: number } {
  return { x: node.position.x + w / 2, y: node.position.y + h / 2 };
}

/** Side of `from` that faces `toward` (cardinal approximation). */
function cardinalSide(dx: number, dy: number): CardinalSide {
  if (Math.abs(dx) >= Math.abs(dy)) {
    return dx >= 0 ? "right" : "left";
  }
  return dy >= 0 ? "bottom" : "top";
}

/** Source handle exits toward the target; target handle faces back toward the source (mutually facing). */
export function handlesForFlowLink(source: Node, target: Node, w: number, h: number): {
  sourceHandle: string;
  targetHandle: string;
} {
  const s = nodeCenter(source, w, h);
  const t = nodeCenter(target, w, h);
  const dx = t.x - s.x;
  const dy = t.y - s.y;
  if (dx === 0 && dy === 0) {
    return { sourceHandle: "out-right", targetHandle: "in-left" };
  }
  const out = cardinalSide(dx, dy);
  const inn = cardinalSide(-dx, -dy);
  return {
    sourceHandle: `out-${out}`,
    targetHandle: `in-${inn}`
  };
}

/** One segment of `R[]`: commodity `T` / `Q` on an edge into or out of a placement. */
export type PiRouteMaterialStub = {
  quantity: number;
  routeTypeId: number;
  /** Short SDE name or `T{id}`. */
  materialLabel: string;
};

export type PiPlacementNodeData = {
  typeId: number;
  structureId: number | null;
  /** 0-based row index in template `P`. */
  index: number;
  /** Template `P[].H` (layer / height hint from exporter). */
  layerH: number;
  /** From SQLite `EveSdeTypes` (via `eveSdeLookupTypes`); null if id missing or SDE not ingested. */
  typeDetails: EveTypeSnapshot | null;
  /** Segments where this placement is the target (material arrives here). */
  incomingRoutes: PiRouteMaterialStub[];
  /** Segments where this placement is the source (material leaves here). */
  outgoingRoutes: PiRouteMaterialStub[];
};

function materialShortLabel(routeTypeId: number, typesById: Record<string, EveTypeSnapshot>): string {
  const snap = typesById[String(routeTypeId)];
  const name = snap?.name?.trim();
  return name && name.length > 0 ? name : `T${routeTypeId}`;
}

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
export function piTemplateToFlowElements(
  template: PiTemplate,
  typesById: Record<string, EveTypeSnapshot> = {}
): {
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

  const incomingByNodeId = new Map<string, PiRouteMaterialStub[]>();
  const outboundByNodeId = new Map<string, PiRouteMaterialStub[]>();

  const nodes: Node<PiPlacementNodeData>[] = pins.map((pin, index) => {
    const x = PADDING + ((pin.Lo - minLo) / spanLo) * innerW;
    const y = PADDING + ((pin.La - minLa) / spanLa) * innerH;
    const key = String(pin.T);
    const typeDetails = typesById[key] ?? null;
    return {
      id: nodeIdForPlacementIndex(index),
      type: "piPlacement",
      position: { x, y },
      data: {
        typeId: pin.T,
        structureId: pin.S,
        index,
        layerH: pin.H,
        typeDetails,
        incomingRoutes: [] as PiRouteMaterialStub[],
        outgoingRoutes: [] as PiRouteMaterialStub[]
      }
    };
  });

  const byStructureId = structureIdToFirstNodeId(pins);

  const nodeById = new Map(nodes.map((n) => [n.id, n]));

  const edgeMarkers = {
    type: MarkerType.ArrowClosed,
    width: 18,
    height: 18
  } as const;

  const edges: Edge[] = [];
  template.R.forEach((route, routeIndex) => {
    const path = route.P;
    if (path.length < 2) return;
    const usePlacementIndex = pathUsesOneBasedPlacementIndices(path, pins.length);
    for (let seg = 0; seg < path.length - 1; seg++) {
      const a = path[seg];
      const b = path[seg + 1];
      let sourceId: string | undefined;
      let targetId: string | undefined;
      if (usePlacementIndex) {
        sourceId = nodeIdForPlacementIndex(a - 1);
        targetId = nodeIdForPlacementIndex(b - 1);
      } else {
        sourceId = byStructureId.get(a);
        targetId = byStructureId.get(b);
      }
      if (!sourceId || !targetId) continue;
      const sourceNode = nodeById.get(sourceId);
      const targetNode = nodeById.get(targetId);
      if (!sourceNode || !targetNode) continue;

      const stub: PiRouteMaterialStub = {
        quantity: route.Q,
        routeTypeId: route.T,
        materialLabel: materialShortLabel(route.T, typesById)
      };
      const outList = outboundByNodeId.get(sourceId) ?? [];
      outList.push(stub);
      outboundByNodeId.set(sourceId, outList);
      const inList = incomingByNodeId.get(targetId) ?? [];
      inList.push(stub);
      incomingByNodeId.set(targetId, inList);

      const { sourceHandle, targetHandle } = handlesForFlowLink(
        sourceNode,
        targetNode,
        PI_NODE_WIDTH,
        PI_NODE_HEIGHT
      );
      edges.push({
        id: `r-${routeIndex}-${seg}-${sourceId}-${targetId}`,
        type: "straight",
        source: sourceId,
        target: targetId,
        sourceHandle,
        targetHandle,
        markerStart: edgeMarkers,
        markerEnd: edgeMarkers
      });
    }
  });

  for (const n of nodes) {
    n.data.incomingRoutes = incomingByNodeId.get(n.id) ?? [];
    n.data.outgoingRoutes = outboundByNodeId.get(n.id) ?? [];
  }

  return { nodes, edges };
}
