// Tiles are 8 bit binary numbers.

const AdjacencyMask = 0b1111; // bits 0-3 = number of adjacent mines
const Mine = 1 << 4; // bit 4 = is there a mine?
const Flag = 1 << 5; // bit 5 = is this flagged?
const Revealed = 1 << 6; // bit 6 = is this revealed?
const Loading = 1 << 7; // bit 7 = is this loading?

const adjacent = tile => tile & AdjacencyMask;
const mine = tile => (tile & Mine) !== 0;
const flag = tile => (tile & Flag) !== 0;
const revealed = tile => (tile & Revealed) !== 0;
const loading = tile => (tile & loading) !== 0;

function withFlag(tile) {
    if (flag(tile)) {
        return tile;
    } else {
        return tile + Flag;
    }
}

function withoutFlag(tile) {
    if (flag(tile)) {
        return tile - Flag;
    } else {
        return tile;
    }
}
