// Tiles are 8-bit binary numbers.

export const AdjacencyMask = 0b1111; // bits 0-3 = number of adjacent mines
export const Mine = 1 << 4; // bit 4 = is there a mine?
export const Flag = 1 << 5; // bit 5 = is this flagged?
export const Revealed = 1 << 6; // bit 6 = is this revealed?

export const adjacent = tile => tile & AdjacencyMask;
export const mine = tile => (tile & Mine) !== 0;
export const flag = tile => (tile & Flag) !== 0;
export const revealed = tile => (tile & Revealed) !== 0;

export function withFlag(tile) {
    if (flag(tile)) {
        return tile;
    } else {
        return tile + Flag;
    }
}

export function withoutFlag(tile) {
    if (flag(tile)) {
        return tile - Flag;
    } else {
        return tile;
    }
}
