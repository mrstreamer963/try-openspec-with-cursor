import { Application, Container, Graphics, Sprite, Text, TextStyle } from 'pixi.js';
import { buildingColorMap, terrainColorMap } from '../content/loadBaseContent';
import type { ContentPack } from '../content/types';
import type { ColonistSnapshot, StateSnapshot, ToolMode } from './types';
import type { SpriteResolver } from './spriteResolver';
import { COLONIST_COLOR, SIM_TICK_MS, TILE_SIZE, WORLD_SIZE } from './types';
import {
  clampTile,
  horizontalVerticalLineTiles,
  rectTiles,
  type TileCoord,
} from './tileShapes';

export type { TileCoord };

export interface SceneLineBuild {
  building: string;
  tiles: TileCoord[];
}

export interface SceneRectDeconstruct {
  tiles: TileCoord[];
}

interface ColonistMotion {
  sampleX: number;
  sampleY: number;
  sampledAt: number;
  vx: number;
  vy: number;
  visualX: number;
  visualY: number;
  initialized: boolean;
}

export interface CameraState {
  offsetX: number;
  offsetY: number;
  zoom: number;
}

export interface ClickTarget {
  kind: 'tile';
  x: number;
  y: number;
}

export interface ColonistClickTarget {
  kind: 'colonist';
  colonist: ColonistSnapshot;
}

export type SceneClick = ClickTarget | ColonistClickTarget;

const COLONIST_LABEL_STYLE = new TextStyle({
  fill: 0xffffff,
  fontSize: 10,
  fontFamily: 'system-ui, sans-serif',
  stroke: { color: 0x1a202c, width: 2 },
  align: 'center',
});

const KEYBOARD_PAN_SPEED = 600;

const PAN_KEY_CODES = new Set([
  'KeyW',
  'KeyA',
  'KeyS',
  'KeyD',
  'ArrowUp',
  'ArrowDown',
  'ArrowLeft',
  'ArrowRight',
]);

const ARROW_KEY_CODES = new Set(['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight']);

const DRAG_CLICK_THRESHOLD_PX = 4;

type DragMode = 'pan' | 'wall-line' | 'deconstruct-rect';

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT';
}

export class PixiRenderer {
  readonly app: Application;
  readonly worldContainer: Container;
  private terrainLayer: Container;
  private buildingsLayer: Container;
  private previewLayer: Container;
  private entitiesLayer: Container;
  private camera: CameraState = { offsetX: 0, offsetY: 0, zoom: 1 };
  private snapshot: StateSnapshot | null = null;
  private terrainDrawn = false;
  private toolMode: ToolMode = null;
  private dragMode: DragMode | null = null;
  private pendingClick = false;
  private dragStart = { x: 0, y: 0 };
  private cameraStart = { offsetX: 0, offsetY: 0 };
  private shapeStartTile: TileCoord | null = null;
  private shapeEndTile: TileCoord | null = null;
  private onSceneClick?: (click: SceneClick) => void;
  private onSceneLineBuild?: (action: SceneLineBuild) => void;
  private onSceneRectDeconstruct?: (action: SceneRectDeconstruct) => void;
  private colonistGraphics = new Map<number, Graphics | Sprite>();
  private colonistLabels = new Map<number, Text>();
  private colonistMotion = new Map<number, ColonistMotion>();
  private pointerDownHandler?: (e: PointerEvent) => void;
  private pointerUpHandler?: (e: PointerEvent) => void;
  private pointerMoveHandler?: (e: PointerEvent) => void;
  private contextMenuHandler?: (e: Event) => void;
  private wheelHandler?: (e: WheelEvent) => void;
  private keydownHandler?: (e: KeyboardEvent) => void;
  private keyupHandler?: (e: KeyboardEvent) => void;
  private blurHandler?: () => void;
  private pressedPanKeys = new Set<string>();
  private applyCameraTicker?: (ticker: { deltaMS: number }) => void;
  private destroyed = false;
  private readonly terrainColors: Record<string, number>;
  private readonly buildingColors: Record<string, number>;
  private readonly berriesPerBush: number;

  constructor(
    private mount: HTMLElement,
    content: ContentPack,
    private readonly spriteResolver: SpriteResolver,
  ) {
    this.terrainColors = terrainColorMap(content);
    this.buildingColors = buildingColorMap(content);
    this.berriesPerBush =
      content.buildings.find((b) => b.id === 'berry_bush')?.on_complete.find((p) => p.type === 'supply')
        ?.amount ?? 3;
    this.app = new Application();
    this.worldContainer = new Container();
    this.terrainLayer = new Container();
    this.buildingsLayer = new Container();
    this.previewLayer = new Container();
    this.entitiesLayer = new Container();
  }

  async init(): Promise<void> {
    await this.app.init({
      resizeTo: this.mount,
      backgroundColor: 0x1a202c,
      antialias: false,
      preference: 'webgl',
    });
    this.mount.appendChild(this.app.canvas);

    this.worldContainer.addChild(this.terrainLayer);
    this.worldContainer.addChild(this.buildingsLayer);
    this.worldContainer.addChild(this.previewLayer);
    this.worldContainer.addChild(this.entitiesLayer);
    this.app.stage.addChild(this.worldContainer);

    this.centerCamera();
    this.setupInteraction();
    this.applyCameraTicker = (ticker) => {
      this.updateKeyboardPan(ticker.deltaMS / 1000);
      this.applyCamera();
    };
    this.app.ticker.add(this.applyCameraTicker);
  }

  destroy(): void {
    if (this.destroyed) return;
    this.destroyed = true;

    const canvas = this.app.canvas;
    if (this.pointerDownHandler) {
      canvas.removeEventListener('pointerdown', this.pointerDownHandler);
    }
    if (this.pointerUpHandler) {
      window.removeEventListener('pointerup', this.pointerUpHandler);
    }
    if (this.pointerMoveHandler) {
      window.removeEventListener('pointermove', this.pointerMoveHandler);
    }
    if (this.contextMenuHandler) {
      canvas.removeEventListener('contextmenu', this.contextMenuHandler);
    }
    if (this.wheelHandler) {
      canvas.removeEventListener('wheel', this.wheelHandler);
    }
    if (this.keydownHandler) {
      window.removeEventListener('keydown', this.keydownHandler);
    }
    if (this.keyupHandler) {
      window.removeEventListener('keyup', this.keyupHandler);
    }
    if (this.blurHandler) {
      window.removeEventListener('blur', this.blurHandler);
    }

    if (this.applyCameraTicker) {
      this.app.ticker.remove(this.applyCameraTicker);
    }

    this.onSceneClick = undefined;
    this.onSceneLineBuild = undefined;
    this.onSceneRectDeconstruct = undefined;
    this.app.destroy(true);
  }

  setToolMode(mode: ToolMode): void {
    this.toolMode = mode;
    this.resetDrag();
    this.clearPreview();
  }

  setOnSceneClick(handler: (click: SceneClick) => void): void {
    this.onSceneClick = handler;
  }

  setOnSceneLineBuild(handler: (action: SceneLineBuild) => void): void {
    this.onSceneLineBuild = handler;
  }

  setOnSceneRectDeconstruct(handler: (action: SceneRectDeconstruct) => void): void {
    this.onSceneRectDeconstruct = handler;
  }

  updateSnapshot(snapshot: StateSnapshot): void {
    this.snapshot = snapshot;
    if (!this.terrainDrawn) {
      this.drawTerrain(snapshot);
      this.terrainDrawn = true;
    }
    this.drawBuildings(snapshot);
    this.syncColonistMotion(snapshot);
    this.drawColonistsFrame();
  }

  /** Advance colonist visuals and redraw entities (call every animation frame). */
  renderFrame(): void {
    this.drawColonistsFrame();
  }

  startRenderLoop(onFrame: () => void): void {
    const loop = () => {
      onFrame();
      requestAnimationFrame(loop);
    };
    requestAnimationFrame(loop);
  }

  private centerCamera(): void {
    const worldW = WORLD_SIZE * TILE_SIZE;
    const worldH = WORLD_SIZE * TILE_SIZE;
    this.camera.offsetX = (this.app.screen.width - worldW) / 2;
    this.camera.offsetY = (this.app.screen.height - worldH) / 2;
    this.applyCamera();
  }

  private applyCamera(): void {
    this.worldContainer.position.set(this.camera.offsetX, this.camera.offsetY);
    this.worldContainer.scale.set(this.camera.zoom);
  }

  private updateKeyboardPan(deltaSeconds: number): void {
    if (this.pressedPanKeys.size === 0) return;

    let dx = 0;
    let dy = 0;
    if (this.pressedPanKeys.has('KeyW') || this.pressedPanKeys.has('ArrowUp')) dy -= 1;
    if (this.pressedPanKeys.has('KeyS') || this.pressedPanKeys.has('ArrowDown')) dy += 1;
    if (this.pressedPanKeys.has('KeyA') || this.pressedPanKeys.has('ArrowLeft')) dx -= 1;
    if (this.pressedPanKeys.has('KeyD') || this.pressedPanKeys.has('ArrowRight')) dx += 1;

    const len = Math.hypot(dx, dy);
    if (len === 0) return;

    const distance = KEYBOARD_PAN_SPEED * deltaSeconds;
    this.camera.offsetX += (dx / len) * distance;
    this.camera.offsetY += (dy / len) * distance;
  }

  private setupInteraction(): void {
    const canvas = this.app.canvas;

    this.contextMenuHandler = (e) => {
      e.preventDefault();
    };
    canvas.addEventListener('contextmenu', this.contextMenuHandler);

    this.pointerDownHandler = (e) => {
      if (e.button !== 0 && e.button !== 2) return;

      this.dragStart = { x: e.clientX, y: e.clientY };
      this.cameraStart = { ...this.camera };
      this.pendingClick = false;
      this.dragMode = null;

      if (e.button === 2) {
        this.dragMode = 'pan';
        canvas.setPointerCapture(e.pointerId);
        return;
      }

      if (this.toolMode === 'wall') {
        const startTile = this.clientToTile(e.clientX, e.clientY);
        if (!startTile) return;
        this.dragMode = 'wall-line';
        this.shapeStartTile = startTile;
        this.shapeEndTile = startTile;
        this.clearPreview();
        canvas.setPointerCapture(e.pointerId);
        return;
      }

      if (this.toolMode === 'deconstruct') {
        const startTile = this.clientToTile(e.clientX, e.clientY);
        if (!startTile) return;
        this.dragMode = 'deconstruct-rect';
        this.shapeStartTile = startTile;
        this.shapeEndTile = startTile;
        this.clearPreview();
        canvas.setPointerCapture(e.pointerId);
        return;
      }

      this.pendingClick = true;
    };
    canvas.addEventListener('pointerdown', this.pointerDownHandler);

    this.pointerUpHandler = (e) => {
      if (e.button !== 0 && e.button !== 2) return;

      const canvas = this.app.canvas;
      if (canvas.hasPointerCapture(e.pointerId)) {
        canvas.releasePointerCapture(e.pointerId);
      }

      const dx = e.clientX - this.dragStart.x;
      const dy = e.clientY - this.dragStart.y;
      const wasDrag = Math.abs(dx) > DRAG_CLICK_THRESHOLD_PX || Math.abs(dy) > DRAG_CLICK_THRESHOLD_PX;
      const mode = this.dragMode;

      if (mode === 'pan') {
        this.resetDrag();
        return;
      }

      if (mode === 'wall-line' && this.shapeStartTile) {
        const endTile = this.clientToTile(e.clientX, e.clientY) ?? this.shapeStartTile;
        const tiles = horizontalVerticalLineTiles(this.shapeStartTile, endTile);
        this.onSceneLineBuild?.({ building: 'wall', tiles });
        this.resetDrag();
        this.clearPreview();
        return;
      }

      if (mode === 'deconstruct-rect' && this.shapeStartTile) {
        const endTile = this.clientToTile(e.clientX, e.clientY) ?? this.shapeStartTile;
        const tiles = rectTiles(this.shapeStartTile, endTile);
        this.onSceneRectDeconstruct?.({ tiles });
        this.resetDrag();
        this.clearPreview();
        return;
      }

      if (this.pendingClick && !wasDrag) {
        this.handleClick(e.clientX, e.clientY);
      }

      this.resetDrag();
    };
    window.addEventListener('pointerup', this.pointerUpHandler);

    this.pointerMoveHandler = (e) => {
      if (this.dragMode === 'pan') {
        const dx = e.clientX - this.dragStart.x;
        const dy = e.clientY - this.dragStart.y;
        this.camera.offsetX = this.cameraStart.offsetX + dx;
        this.camera.offsetY = this.cameraStart.offsetY + dy;
        this.applyCamera();
        return;
      }

      if (this.dragMode === 'wall-line' && this.shapeStartTile) {
        this.shapeEndTile = this.clientToTile(e.clientX, e.clientY) ?? this.shapeStartTile;
        this.drawWallLinePreview(this.shapeStartTile, this.shapeEndTile);
        return;
      }

      if (this.dragMode === 'deconstruct-rect' && this.shapeStartTile) {
        this.shapeEndTile = this.clientToTile(e.clientX, e.clientY) ?? this.shapeStartTile;
        this.drawDeconstructRectPreview(this.shapeStartTile, this.shapeEndTile);
      }
    };
    window.addEventListener('pointermove', this.pointerMoveHandler);

    this.wheelHandler = (e) => {
      e.preventDefault();
      const rect = canvas.getBoundingClientRect();
      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;

      const worldX = (mouseX - this.camera.offsetX) / this.camera.zoom;
      const worldY = (mouseY - this.camera.offsetY) / this.camera.zoom;

      const factor = e.deltaY < 0 ? 1.1 : 0.9;
      const newZoom = Math.min(3, Math.max(0.3, this.camera.zoom * factor));

      this.camera.offsetX = mouseX - worldX * newZoom;
      this.camera.offsetY = mouseY - worldY * newZoom;
      this.camera.zoom = newZoom;
      this.applyCamera();
    };
    canvas.addEventListener('wheel', this.wheelHandler, { passive: false });

    this.keydownHandler = (e) => {
      if (isEditableTarget(e.target)) return;
      if (!PAN_KEY_CODES.has(e.code)) return;
      if (ARROW_KEY_CODES.has(e.code)) e.preventDefault();
      this.pressedPanKeys.add(e.code);
    };
    window.addEventListener('keydown', this.keydownHandler);

    this.keyupHandler = (e) => {
      this.pressedPanKeys.delete(e.code);
    };
    window.addEventListener('keyup', this.keyupHandler);

    this.blurHandler = () => {
      this.pressedPanKeys.clear();
    };
    window.addEventListener('blur', this.blurHandler);
  }

  private resetDrag(): void {
    this.dragMode = null;
    this.pendingClick = false;
    this.shapeStartTile = null;
    this.shapeEndTile = null;
  }

  private clearPreview(): void {
    this.previewLayer.removeChildren();
  }

  private clientToTile(clientX: number, clientY: number): TileCoord | null {
    const rect = this.app.canvas.getBoundingClientRect();
    const localX = (clientX - rect.left - this.camera.offsetX) / this.camera.zoom;
    const localY = (clientY - rect.top - this.camera.offsetY) / this.camera.zoom;
    const tileX = Math.floor(localX / TILE_SIZE);
    const tileY = Math.floor(localY / TILE_SIZE);

    if (tileX < 0 || tileY < 0 || tileX >= WORLD_SIZE || tileY >= WORLD_SIZE) {
      return null;
    }

    return clampTile(tileX, tileY, WORLD_SIZE);
  }

  private drawWallLinePreview(start: TileCoord, end: TileCoord): void {
    this.clearPreview();
    const color = this.buildingColors.wall ?? 0x718096;
    for (const tile of horizontalVerticalLineTiles(start, end)) {
      this.addTileGraphic(this.previewLayer, tile.x, tile.y, null, color, 0.45, 2);
    }
  }

  private drawDeconstructRectPreview(start: TileCoord, end: TileCoord): void {
    this.clearPreview();
    for (const tile of rectTiles(start, end)) {
      const g = new Graphics();
      const pad = 2;
      g.rect(
        tile.x * TILE_SIZE + pad,
        tile.y * TILE_SIZE + pad,
        TILE_SIZE - pad * 2,
        TILE_SIZE - pad * 2,
      );
      g.fill({ color: 0xe53e3e, alpha: 0.45 });
      this.previewLayer.addChild(g);
    }
  }

  private handleClick(clientX: number, clientY: number): void {
    if (!this.onSceneClick) return;
    const tile = this.clientToTile(clientX, clientY);
    if (!tile) return;

    const { x: tileX, y: tileY } = tile;
    const rect = this.app.canvas.getBoundingClientRect();
    const localX = (clientX - rect.left - this.camera.offsetX) / this.camera.zoom;
    const localY = (clientY - rect.top - this.camera.offsetY) / this.camera.zoom;

    if (this.toolMode === null) {
      const hitRadius = TILE_SIZE * 0.35;
      const colonist = this.snapshot?.colonists.find((c) => {
      const motion = this.colonistMotion.get(c.id);
      const wx = motion?.visualX ?? c.x;
      const wy = motion?.visualY ?? c.y;
      const cx = wx * TILE_SIZE + TILE_SIZE / 2;
      const cy = wy * TILE_SIZE + TILE_SIZE / 2;
      const dx = localX - cx;
      const dy = localY - cy;
      return dx * dx + dy * dy <= hitRadius * hitRadius;
    });
      if (colonist) {
        this.onSceneClick({ kind: 'colonist', colonist });
        return;
      }
    }

    this.onSceneClick({ kind: 'tile', x: tileX, y: tileY });
  }

  private drawTerrain(snapshot: StateSnapshot): void {
    this.terrainLayer.removeChildren();
    for (const tile of snapshot.tiles) {
      const texture = this.spriteResolver.resolveTerrain(tile.terrain);
      const color = this.terrainColors[tile.terrain] ?? 0x4a5568;
      this.addTileGraphic(this.terrainLayer, tile.x, tile.y, texture, color);
    }
  }

  private addTileGraphic(
    layer: Container,
    x: number,
    y: number,
    texture: import('pixi.js').Texture | null,
    color: number,
    alpha = 1,
    inset = 0,
  ): void {
    const px = x * TILE_SIZE + inset;
    const py = y * TILE_SIZE + inset;
    const size = TILE_SIZE - inset * 2;
    if (texture) {
      const sprite = new Sprite({ texture, roundPixels: true });
      sprite.x = px;
      sprite.y = py;
      sprite.width = size;
      sprite.height = size;
      sprite.alpha = alpha;
      layer.addChild(sprite);
      return;
    }
    const g = new Graphics();
    g.rect(px, py, size, size);
    g.fill({ color, alpha });
    layer.addChild(g);
  }

  private drawBuildings(snapshot: StateSnapshot): void {
    this.buildingsLayer.removeChildren();
    for (const site of snapshot.construction_sites ?? []) {
      const texture = this.spriteResolver.resolveBuilding(site.building);
      const color = this.buildingColors[site.building] ?? 0x718096;
      const alpha = 0.25 + site.progress * 0.45;
      this.addTileGraphic(this.buildingsLayer, site.x, site.y, texture, color, alpha, 2);
      this.drawConstructionProgressBar(site.x, site.y, site.progress);
    }
    for (const b of snapshot.buildings) {
      const texture = this.spriteResolver.resolveBuilding(b.building);
      let color = this.buildingColors[b.building] ?? 0x718096;
      let alpha = 1;
      if (b.building === 'berry_bush' && b.berries != null) {
        alpha = 0.45 + (b.berries / this.berriesPerBush) * 0.55;
      }
      this.addTileGraphic(this.buildingsLayer, b.x, b.y, texture, color, alpha, 2);
    }
    for (const site of snapshot.deconstruction_sites ?? []) {
      const g = new Graphics();
      const pad = 2;
      g.rect(
        site.x * TILE_SIZE + pad,
        site.y * TILE_SIZE + pad,
        TILE_SIZE - pad * 2,
        TILE_SIZE - pad * 2,
      );
      const alpha = 0.25 + site.progress * 0.45;
      g.fill({ color: 0xe53e3e, alpha });
      this.buildingsLayer.addChild(g);
      this.drawDeconstructionProgressBar(site.x, site.y, site.progress);
    }
  }

  private drawConstructionProgressBar(tileX: number, tileY: number, progress: number): void {
    const barWidth = TILE_SIZE - 4;
    const barHeight = 4;
    const x = tileX * TILE_SIZE + 2;
    const y = tileY * TILE_SIZE - barHeight - 2;
    const p = Math.min(1, Math.max(0, progress));

    const bar = new Graphics();
    bar.rect(x, y, barWidth, barHeight);
    bar.fill({ color: 0x1a202c, alpha: 0.75 });
    if (p > 0) {
      bar.rect(x, y, barWidth * p, barHeight);
      bar.fill({ color: 0x48bb78, alpha: 1 });
    }
    bar.rect(x, y, barWidth, barHeight);
    bar.stroke({ color: 0xffffff, width: 1, alpha: 0.65 });
    this.buildingsLayer.addChild(bar);
  }

  private drawDeconstructionProgressBar(tileX: number, tileY: number, progress: number): void {
    const barWidth = TILE_SIZE - 4;
    const barHeight = 4;
    const x = tileX * TILE_SIZE + 2;
    const y = tileY * TILE_SIZE - barHeight - 2;
    const p = Math.min(1, Math.max(0, progress));

    const bar = new Graphics();
    bar.rect(x, y, barWidth, barHeight);
    bar.fill({ color: 0x1a202c, alpha: 0.75 });
    if (p > 0) {
      bar.rect(x, y, barWidth * p, barHeight);
      bar.fill({ color: 0xe53e3e, alpha: 1 });
    }
    bar.rect(x, y, barWidth, barHeight);
    bar.stroke({ color: 0xffffff, width: 1, alpha: 0.65 });
    this.buildingsLayer.addChild(bar);
  }

  private syncColonistMotion(snapshot: StateSnapshot): void {
    const now = performance.now();
    const seen = new Set<number>();

    for (const c of snapshot.colonists) {
      seen.add(c.id);
      let motion = this.colonistMotion.get(c.id);
      if (!motion) {
        motion = {
          sampleX: c.x,
          sampleY: c.y,
          sampledAt: now,
          vx: 0,
          vy: 0,
          visualX: c.x,
          visualY: c.y,
          initialized: false,
        };
        this.colonistMotion.set(c.id, motion);
      }

      if (motion.initialized) {
        const dt = (now - motion.sampledAt) / 1000;
        if (dt > 0 && !c.at_task_stand) {
          motion.vx = (c.x - motion.sampleX) / dt;
          motion.vy = (c.y - motion.sampleY) / dt;
        } else {
          motion.vx = 0;
          motion.vy = 0;
        }
      } else {
        motion.visualX = c.x;
        motion.visualY = c.y;
        motion.initialized = true;
      }

      motion.sampleX = c.x;
      motion.sampleY = c.y;
      motion.sampledAt = now;
    }

    for (const id of this.colonistMotion.keys()) {
      if (!seen.has(id)) {
        this.colonistMotion.delete(id);
      }
    }
  }

  private drawColonistsFrame(): void {
    if (!this.snapshot) return;

    const now = performance.now();
    const paused = this.snapshot.paused;
    const seen = new Set<number>();

    const colonistTexture = this.spriteResolver.resolveEntity('colonist');

    for (const c of this.snapshot.colonists) {
      seen.add(c.id);
      const motion = this.colonistMotion.get(c.id);
      if (!motion) continue;

      if (paused) {
        motion.visualX = motion.sampleX;
        motion.visualY = motion.sampleY;
        motion.vx = 0;
        motion.vy = 0;
      } else if (c.at_task_stand) {
        motion.visualX = motion.sampleX;
        motion.visualY = motion.sampleY;
        motion.vx = 0;
        motion.vy = 0;
      } else {
        const elapsed = (now - motion.sampledAt) / 1000;
        // Extrapolate between 20 Hz snapshots so rAF draws smooth motion.
        const speed = Math.max(1, this.snapshot.speed);
        const maxExtrapolate = ((SIM_TICK_MS / 1000) * 1.25) / speed;
        const t = Math.min(elapsed, maxExtrapolate);
        motion.visualX = motion.sampleX + motion.vx * t;
        motion.visualY = motion.sampleY + motion.vy * t;
      }

      let graphic = this.colonistGraphics.get(c.id);
      const cx = motion.visualX * TILE_SIZE + TILE_SIZE / 2;
      const cy = motion.visualY * TILE_SIZE + TILE_SIZE / 2;

      if (colonistTexture) {
        if (!(graphic instanceof Sprite)) {
          if (graphic) {
            this.entitiesLayer.removeChild(graphic);
            this.colonistGraphics.delete(c.id);
          }
          const sprite = new Sprite(colonistTexture);
          sprite.anchor.set(0.5, 0.5);
          this.colonistGraphics.set(c.id, sprite);
          this.entitiesLayer.addChild(sprite);
          graphic = sprite;
        }
        graphic.x = cx;
        graphic.y = cy;
        graphic.width = TILE_SIZE * 0.7;
        graphic.height = TILE_SIZE * 0.7;
      } else {
        if (!(graphic instanceof Graphics)) {
          if (graphic) {
            this.entitiesLayer.removeChild(graphic);
            this.colonistGraphics.delete(c.id);
          }
          const g = new Graphics();
          this.colonistGraphics.set(c.id, g);
          this.entitiesLayer.addChild(g);
          graphic = g;
        }
        graphic.clear();
        graphic.circle(cx, cy, TILE_SIZE * 0.35);
        graphic.fill(COLONIST_COLOR);
      }

      let label = this.colonistLabels.get(c.id);
      if (!label) {
        label = new Text({ text: c.name, style: COLONIST_LABEL_STYLE });
        label.anchor.set(0.5, 1);
        this.colonistLabels.set(c.id, label);
        this.entitiesLayer.addChild(label);
      } else if (label.text !== c.name) {
        label.text = c.name;
      }
      label.x = cx;
      label.y = cy - TILE_SIZE * 0.35 - 2;
    }

    for (const [id, graphic] of this.colonistGraphics) {
      if (!seen.has(id)) {
        this.entitiesLayer.removeChild(graphic);
        this.colonistGraphics.delete(id);
      }
    }

    for (const [id, label] of this.colonistLabels) {
      if (!seen.has(id)) {
        this.entitiesLayer.removeChild(label);
        this.colonistLabels.delete(id);
      }
    }
  }
}
