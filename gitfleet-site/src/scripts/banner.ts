import { bannerSplitIndex } from "@/data/site";

export const BANNER_CELL_WIDTH = 8;
export const BANNER_CELL_HEIGHT = 16;

const CELL_W = BANNER_CELL_WIDTH;
const CELL_H = BANNER_CELL_HEIGHT;
const STROKE = 2;
const BLOCK_INSET = 0.2;

type CellRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export type BannerGlyphPath = {
  d: string;
  x: number;
  y: number;
  accent: boolean;
};

function rectPath(rect: CellRect): string {
  return `M${rect.x} ${rect.y}h${rect.width}v${rect.height}h${-rect.width}z`;
}

function fullBlock(): CellRect[] {
  return [
    {
      x: 0,
      y: BLOCK_INSET,
      width: CELL_W,
      height: CELL_H - BLOCK_INSET * 2,
    },
  ];
}

function horizontalBar(): CellRect[] {
  return [
    {
      x: 0,
      y: CELL_H / 2 - STROKE,
      width: CELL_W,
      height: STROKE * 2,
    },
  ];
}

function verticalBar(): CellRect[] {
  return [
    {
      x: CELL_W / 2 - STROKE,
      y: 0,
      width: STROKE * 2,
      height: CELL_H,
    },
  ];
}

function corner(horizontalLeft: boolean, verticalUp: boolean): CellRect[] {
  const centerX = CELL_W / 2;
  const centerY = CELL_H / 2;

  return [
    {
      x: horizontalLeft ? 0 : centerX - STROKE,
      y: centerY - STROKE,
      width: horizontalLeft ? centerX + STROKE : CELL_W - (centerX - STROKE),
      height: STROKE * 2,
    },
    {
      x: centerX - STROKE,
      y: verticalUp ? 0 : centerY - STROKE,
      width: STROKE * 2,
      height: verticalUp ? centerY + STROKE : CELL_H - (centerY - STROKE),
    },
  ];
}

const GLYPH_RECTS: Record<string, CellRect[]> = {
  "█": fullBlock(),
  "═": horizontalBar(),
  "║": verticalBar(),
  "╔": corner(false, false),
  "╗": corner(true, false),
  "╚": corner(false, true),
  "╝": corner(true, true),
};

export function bannerGlyphRects(character: string): CellRect[] {
  return GLYPH_RECTS[character] ?? [];
}

export function bannerViewBox(columnCount: number, rowCount: number): string {
  return `0 0 ${columnCount * CELL_W} ${rowCount * CELL_H}`;
}

export function bannerGlyphPaths(lines: readonly string[]): BannerGlyphPath[] {
  const paths: BannerGlyphPath[] = [];

  lines.forEach((line, row) => {
    [...line].forEach((character, column) => {
      const rects = bannerGlyphRects(character);
      if (rects.length === 0) {
        return;
      }

      paths.push({
        d: rects.map(rectPath).join(""),
        x: column * CELL_W,
        y: row * CELL_H,
        accent: column >= bannerSplitIndex,
      });
    });
  });

  return paths;
}
