import { Application, Container, Graphics } from 'pixi.js';
import type { ColonistSnapshot, StateSnapshot } from './types';
import {
  BUILDING_COLORS,
  COLONIST_COLOR,
  TERRAIN_COLORS,
  TILE_SIZE,
  WORLD_SIZE,
} from './types';

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
    this.app.ticker.add(() => this.applyCamera());
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
    this.drawColonists(snapshot);
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

    canvas.addEventListener('pointerdown', (e) => {
      this.isDragging = true;
      this.dragStart = { x: e.clientX, y: e.clientY };
      this.cameraStart = { ...this.camera };
    });

    window.addEventListener('pointerup', (e) => {
      if (!this.isDragging) return;
      const dx = e.clientX - this.dragStart.x;
      const dy = e.clientY - this.dragStart.y;
      const wasDrag = Math.abs(dx) > 4 || Math.abs(dy) > 4;
      this.isDragging = false;
      if (!wasDrag) {
        this.handleClick(e.clientX, e.clientY);
      }
    });

    window.addEventListener('pointermove', (e) => {
      if (!this.isDragging) return;
      const dx = e.clientX - this.dragStart.x;
      const dy = e.clientY - this.dragStart.y;
      this.camera.offsetX = this.cameraStart.offsetX + dx;
      this.camera.offsetY = this.cameraStart.offsetY + dy;
      this.applyCamera();
    });

    canvas.addEventListener(
      'wheel',
      (e) => {
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
      },
      { passive: false },
    );
  }

  private handleClick(clientX: number, clientY: number): void {
    if (!this.onSceneClick) return;
    const rect = this.app.canvas.getBoundingClientRect();
    const localX = (clientX - rect.left - this.camera.offsetX) / this.camera.zoom;
    const localY = (clientY - rect.top - this.camera.offsetY) / this.camera.zoom;
    const tileX = Math.floor(localX / TILE_SIZE);
    const tileY = Math.floor(localY / TILE_SIZE);

    if (tileX < 0 || tileY < 0 || tileX >= WORLD_SIZE || tileY >= WORLD_SIZE) return;

    const colonist = this.snapshot?.colonists.find((c) => c.x === tileX && c.y === tileY);
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
      g.fill(BUILDING_COLORS[b.building]);
      this.buildingsLayer.addChild(g);
    }
  }

  private drawColonists(snapshot: StateSnapshot): void {
    const seen = new Set<number>();
    for (const c of snapshot.colonists) {
      seen.add(c.id);
      let g = this.colonistGraphics.get(c.id);
      if (!g) {
        g = new Graphics();
        this.colonistGraphics.set(c.id, g);
        this.entitiesLayer.addChild(g);
      }
      g.clear();
      const cx = c.x * TILE_SIZE + TILE_SIZE / 2;
      const cy = c.y * TILE_SIZE + TILE_SIZE / 2;
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
