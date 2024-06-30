class ChunkStore {
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
}