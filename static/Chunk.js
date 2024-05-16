const chunkSize = 16;
class Chunk {
    coords;
    tiles;
    canvas;

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
     * @param socket {WebSocket}
     */
    send(socket) {
        socket.send(this.serialize());
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

    /**
     * The inverse of indexOf()
     * @param index {number}
     * @returns {number[]}
     */
    coordsOf(index) {
        const col = index % chunkSize;
        const row = Math.floor(index/chunkSize);
        return vectorAdd(this.topLeft(), [col, row]);
    }

    updateTile(worldCoords, tile) {
        const index = this.indexOf(worldCoords);
        if (index === -1) return;

        this.tiles[index] = tile;

        this.redraw = true;
    }

    getTile(worldCoords) {
        const index = this.indexOf(worldCoords);
        if (index === -1) return;

        return this.tiles[index];
    }

    /**
     * Draw this to the canvas context
     * @param context {CanvasRenderingContext2D}
     * @param screenX {number}
     * @param screenY {number}
     * @param drawFunction {function}
     */
    draw(context, [screenX, screenY], drawFunction) {
        // Redraw this chunk if we need to
        if (this.redraw) {
            drawFunction(this);
        }

        context.drawImage(this.canvas, screenX, screenY);
    }

    addMine(index, chunkStore) {
        if (index < 0 || index > chunkSize*chunkSize) return;

        const tileIsMineAlready = mine(this.tiles[index]);
        if (tileIsMineAlready) return;

        this.tiles[index] |= Mine;

        // Now we need to update the number of adjacent tiles
        forEachNeighbour(this.coordsOf(index), (coordsOfAdjTile) => {
            const indexOfAdjTile = this.indexOf(coordsOfAdjTile);
            if (indexOfAdjTile === -1) {
                const adjChunk = chunkStore.getChunk(coordsOfAdjTile);
                if (adjChunk) {
                    const adjIndex = adjChunk.indexOf(coordsOfAdjTile);
                    adjChunk.tiles[adjIndex] += 1;
                }
            } else {
                this.tiles[indexOfAdjTile] += 1;
            }
        });
    }

    /**
     *
     * @param player {Player}
     * @param worldCoords {number[]}
     * @param world {World}
     */
    reveal(player, worldCoords, world) {
        const index = this.indexOf(worldCoords);
        if (index === -1) {
            world.queueReveal(player, worldCoords);
            return;
        }

        const tile = this.tiles[index];

        if (revealed(tile)) return;

        const numberOfAdjacentMines = adjacent(tile);
        player.hasRevealed(tile, world);
        this.tiles[index] += Revealed;

        // Reveal adjacent tiles if none of them are mines
        if (numberOfAdjacentMines === 0) {
            forEachNeighbour(worldCoords, (adjacentCoords) => {
                const adjacentTile = this.getTile(adjacentCoords);
                if (!revealed(adjacentTile)) {
                    this.reveal(player, adjacentCoords, world);
                }
            });
        }
        else {
        }
    }

    flag(worldCoords) {
        const index = this.indexOf(worldCoords);
        if (index === -1) return;

        const tile = this.tiles[index];
        if (flag(tile)) {
            this.tiles[index] -= Flag;
        } else {
            this.tiles[index] += Flag;
        }
    }

    topLeft() {
        return this.coords;
    }
    topRight() {
        return vectorAdd(this.topLeft(), [chunkSize, 0]);
    }
    bottomLeft() {
        return vectorAdd(this.topLeft(), [0, chunkSize]);
    }
    bottomRight() {
        return vectorAdd(this.topLeft(), [chunkSize, chunkSize]);
    }
    rect() {
        return [this.topLeft(), this.bottomRight()];
    }

    publicVersion() {
        const publicTiles = this.tiles.map(tile => publicVersion(tile));
        return {
            coords: this.coords,
            tiles: publicTiles
        };
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