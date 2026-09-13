import { describe, expect, it } from "vitest";

import { bannerLines, bannerSplitIndex } from "@/data/site";
import {
  BANNER_CELL_HEIGHT,
  BANNER_CELL_WIDTH,
  bannerGlyphPaths,
  bannerGlyphRects,
  bannerViewBox,
} from "@/scripts/banner";

describe("banner geometry", () => {
  it("keeps every banner row the same width", () => {
    const widths = new Set(bannerLines.map((line) => line.length));

    expect(widths).toEqual(new Set([62]));
    expect(bannerSplitIndex).toBe(21);
  });

  it("covers every non-space banner glyph with vector cells", () => {
    const characters = new Set(bannerLines.join(""));
    characters.delete(" ");

    for (const character of characters) {
      expect(bannerGlyphRects(character).length).toBeGreaterThan(0);
    }

    expect(bannerGlyphRects(" ")).toEqual([]);
    expect(bannerGlyphRects("A")).toEqual([]);
  });

  it("places glyphs on a uniform 8 by 16 grid", () => {
    const paths = bannerGlyphPaths(bannerLines);
    const splitX = bannerSplitIndex * BANNER_CELL_WIDTH;

    expect(bannerViewBox(62, 6)).toBe("0 0 496 96");
    expect(paths.length).toBeGreaterThan(100);
    expect(paths.every((path) => path.d.startsWith("M"))).toBe(true);
    expect(paths.filter((path) => !path.accent).every((path) => path.x < splitX)).toBe(true);
    expect(paths.filter((path) => path.accent).every((path) => path.x >= splitX)).toBe(true);
    expect(paths.at(-1)?.y).toBe(5 * BANNER_CELL_HEIGHT);
  });
});
