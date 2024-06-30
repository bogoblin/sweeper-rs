const chunkSize = 16;
class Chunk {
    coords;
    tiles;
    canvas;
    redraw = true;

    constructor(coords, tiles) {
        this.coords = chunkCoords(coords);
        if (tiles) {
            this.tiles = Uint8Array.from(tiles);
        }
        else {
            this.tiles = new Uint8Array(chunkSize*chunkSize);
        }

        this.redraw = true; // This chunk needs to be drawn again
    }

    /**
     * Chunk stores an array of tiles. This function returns the index of that array that corresponds to the given world coordinates.
     * @param worldCoords {number[]} the coordinates to find the index of.
     * @returns {number} the index for the tiles array for this coordinate. Returns -1 if the coordinate is not in this chunk.
     */
    indexOf(worldCoords) {
        const row = Math.floor(worldCoords[1]) - this.coords[1];
        const col = Math.floor(worldCoords[0]) - this.coords[0];
        if (row >= chunkSize || col >= chunkSize || row < 0 || col < 0) {
            return -1;
        }
        return row*chunkSize + col;
    }

    updateTile(worldCoords, tile) {
        const index = this.indexOf(worldCoords);
        if (index === -1) return;

        this.tiles[index] = tile;

        this.redraw = true;
    }
}

/**
 *
 * @param x {number}
 * @param y {number}
 * @returns {number[]}
 */
const chunkCoords = ([x,y]) => {
    return [
        Math.floor(x/chunkSize)*chunkSize,
        Math.floor(y/chunkSize)*chunkSize
    ];
}

const chunkKey = (worldCoords) => {
    const worldTopLeft = chunkCoords(worldCoords);
    return `${worldTopLeft[0]},${worldTopLeft[1]}`;
}

const defaultChunk = new Chunk([0,0]);