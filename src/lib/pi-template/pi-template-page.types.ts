import type { EveTypeSnapshot } from "$models/domain";
import type { PiRouteMaterialStub } from "$lib/pi-template/template-to-flow";

/** Display model for the template summary grid (derived on the PI Templates page). */
export interface PiTemplateSummaryStats {
  cmdCtrLv: number;
  diam: number;
  plnId: number;
  plnTypeName: string | null;
  placements: number;
  routes: number;
  links: number;
  totalStructureCostIsk: number;
  placementsMissingBasePrice: number;
}

/** Resolved selection for the structure details aside. */
export interface SelectedPlacementDetail {
  typeId: number;
  structureId: number | null;
  index: number;
  layerH: number;
  typeDetails: EveTypeSnapshot | null;
  incomingRoutes: PiRouteMaterialStub[];
  outgoingRoutes: PiRouteMaterialStub[];
}
