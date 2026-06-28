import { Application, Container, Graphics } from 'pixi.js';
import type { ColonistSnapshot, StateSnapshot } from './types';
import {
  BUILDING_COLORS,
  COLONIST_COLOR,
  SIM_TICK_MS,
  TERRAIN_COLORS,
  TILE_SIZE,
  WORLD_SIZE,
} from './types';

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

export class PixiRenderer {
  readonly app: Application;
  readonly worldContainer: Container;
  private terrainLayer: Container;
  private buildingsLayer: Container;
  private entitiesLayer: Container;
  private camera: CameraState = { offsetX: 0, offsetY: 0, zoom: 1 };
  private snapshot: StateSnapshot | null = null;
  private terrainDrawn = false;
  private isDragging = false;
  private dragStart = { x: 0, y: 0 };
  private cameraStart = { offsetX: 0, offsetY: 0 };
  private onSceneClick?: (click: SceneClick) => void;
  private colonistGraphics = new Map<number, Graphics>();
  private colonistMotion = new Map<number, ColonistMotion>();
  private pointerDownHandler?: (e: PointerEvent) => void;
  private pointerUpHandler?: (e: PointerEvent) => void;
  private pointerMoveHandler?: (e: PointerEvent) => void;
  private wheelHandler?: (e: WheelEvent) => void;
  private applyCameraTicker?: () => void;
  private destroyed = false;

  constructor(private mount: HTMLElement) {
    this.app = new Application();
    this.worldContainer = new Container();
    this.terrainLayer = new Container();
    this.buildingsLayer = new Container();
    this.entitiesLayer = new Container();
  }

  async init(): Promise<void> {
    await this.app.init({
      resizeTo: this.mount,
      backgroundColor: 0x1a202c,
      antialias: true,
    });
    this.mount.appendChild(this.app.canvas);

    this.worldContainer.addChild(this.terrainLayer);
    this.worldContainer.addChild(this.buildingsLayer);
    this.worldContainer.addChild(this.entitiesLayer);
    this.app.stage.addChild(this.worldContainer);

    this.centerCamera();
    this.setupInteraction();
    this.applyCameraTicker = () => this.applyCamera();
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
    if (this.wheelHandler) {
      canvas.removeEventListener('wheel', this.wheelHandler);
    }

    if (this.applyCameraTicker) {
      this.app.ticker.remove(this.applyCameraTicker);
    }

    this.onSceneClick = undefined;
    this.app.destroy(true);
  }

  setOnSceneClick(handler: (click: SceneClick) => void): void {
    this.onSceneClick = handler;
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

  private setupInteraction(): void {
    const canvas = this.app.canvas;

    this.pointerDownHandler = (e) => {
      this.isDragging = true;
      this.dragStart = { x: e.clientX, y: e.clientY };
      this.cameraStart = { ...this.camera };
    };
    canvas.addEventListener('pointerdown', this.pointerDownHandler);

    this.pointerUpHandler = (e) => {
      if (!this.isDragging) return;
      const dx = e.clientX - this.dragStart.x;
      const dy = e.clientY - this.dragStart.y;
      const wasDrag = Math.abs(dx) > 4 || Math.abs(dy) > 4;
      this.isDragging = false;
      if (!wasDrag) {
        this.handleClick(e.clientX, e.clientY);
      }
    };
    window.addEventListener('pointerup', this.pointerUpHandler);

    this.pointerMoveHandler = (e) => {
      if (!this.isDragging) return;
      const dx = e.clientX - this.dragStart.x;
      const dy = e.clientY - this.dragStart.y;
      this.camera.offsetX = this.cameraStart.offsetX + dx;
      this.camera.offsetY = this.cameraStart.offsetY + dy;
      this.applyCamera();
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
  }

  private handleClick(clientX: number, clientY: number): void {
    if (!this.onSceneClick) return;
    const rect = this.app.canvas.getBoundingClientRect();
    const localX = (clientX - rect.left - this.camera.offsetX) / this.camera.zoom;
    const localY = (clientY - rect.top - this.camera.offsetY) / this.camera.zoom;
    const tileX = Math.floor(localX / TILE_SIZE);
    const tileY = Math.floor(localY / TILE_SIZE);

    if (tileX < 0 || tileY < 0 || tileX >= WORLD_SIZE || tileY >= WORLD_SIZE) return;

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

    this.onSceneClick({ kind: 'tile', x: tileX, y: tileY });
  }

  private drawTerrain(snapshot: StateSnapshot): void {
    this.terrainLayer.removeChildren();
    for (const tile of snapshot.tiles) {
      const g = new Graphics();
      g.rect(tile.x * TILE_SIZE, tile.y * TILE_SIZE, TILE_SIZE, TILE_SIZE);
      g.fill(TERRAIN_COLORS[tile.terrain]);
      this.terrainLayer.addChild(g);
    }
  }

  private drawBuildings(snapshot: StateSnapshot): void {
    this.buildingsLayer.removeChildren();
    for (const b of snapshot.buildings) {
      const g = new Graphics();
      const pad = 2;
      g.rect(
        b.x * TILE_SIZE + pad,
        b.y * TILE_SIZE + pad,
        TILE_SIZE - pad * 2,
        TILE_SIZE - pad * 2,
      );
      let color = BUILDING_COLORS[b.building];
      let alpha = 1;
      if (b.building === 'BerryBush' && b.berries != null) {
        alpha = 0.45 + (b.berries / 3) * 0.55;
      }
      g.fill({ color, alpha });
      this.buildingsLayer.addChild(g);
    }
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
        if (dt > 0) {
          motion.vx = (c.x - motion.sampleX) / dt;
          motion.vy = (c.y - motion.sampleY) / dt;
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

    for (const c of this.snapshot.colonists) {
      seen.add(c.id);
      const motion = this.colonistMotion.get(c.id);
      if (!motion) continue;

      if (paused) {
        motion.visualX = motion.sampleX;
        motion.visualY = motion.sampleY;
        motion.vx = 0;
        motion.vy = 0;
      } else {
        const elapsed = (now - motion.sampledAt) / 1000;
        // Extrapolate between 20 Hz snapshots so rAF draws smooth motion.
        const maxExtrapolate = (SIM_TICK_MS / 1000) * 1.25;
        const t = Math.min(elapsed, maxExtrapolate);
        motion.visualX = motion.sampleX + motion.vx * t;
        motion.visualY = motion.sampleY + motion.vy * t;
      }

      let g = this.colonistGraphics.get(c.id);
      if (!g) {
        g = new Graphics();
        this.colonistGraphics.set(c.id, g);
        this.entitiesLayer.addChild(g);
      }
      g.clear();
      const cx = motion.visualX * TILE_SIZE + TILE_SIZE / 2;
      const cy = motion.visualY * TILE_SIZE + TILE_SIZE / 2;
      g.circle(cx, cy, TILE_SIZE * 0.35);
      g.fill(COLONIST_COLOR);
    }

    for (const [id, g] of this.colonistGraphics) {
      if (!seen.has(id)) {
        this.entitiesLayer.removeChild(g);
        this.colonistGraphics.delete(id);
      }
    }
  }
}
