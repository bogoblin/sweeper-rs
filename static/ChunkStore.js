import {Chunk, chunkKey} from "./Chunk.js";
import {mine, revealed} from "./Tile.js";

export class ChunkStore {
    constructor() {
        this.chunks = {};
    }

    /**
     * Add a chunk to the chunk store
     * @param chunk {Chunk}
     */
    addChunk(chunk) {
        this.chunks[chunkKey(chunk.coords)] = chunk;
        return chunk;
    }

    /**
     * Get the chunk that the given coordinates lies in
     * @param worldCoords {number[]}
     * @returns {Chunk}
     */
    getChunk(worldCoords) {
        return this.chunks[chunkKey(worldCoords)];
    }

    getOrCreateChunk(worldCoords) {
        let chunk = this.getChunk(worldCoords);
        if (!chunk) {
            return this.addChunk(new Chunk(worldCoords));
        }
        return chunk;
    }

    getTile(worldCoords) {
        return this.getOrCreateChunk(worldCoords).getTile(worldCoords);
    }

    updateTile(worldCoords, tile) {
        this.getOrCreateChunk(worldCoords).updateTile(worldCoords, tile);
    }

    updateRect([leftX, topY], updated) {
        // TODO: I don't really like the way this function returns a boolean like that
        let dead = false;
        for (let relative_x = 0; relative_x < updated.length; relative_x++) {
            for (let relative_y = 0; relative_y < updated[0].length; relative_y++) {
                let x = relative_x + leftX;
                let y = relative_y + topY;
                const updatedTile = updated[relative_x][relative_y];
                if (updatedTile !== 0) {
                    // Optimization: Can skip some chunk lookup here
                    this.updateTile([x, y], updatedTile);
                    if (revealed(updatedTile) && mine(updatedTile)) {
                        dead = true;
                    }
                }
            }
        }
        return dead;
    }
}