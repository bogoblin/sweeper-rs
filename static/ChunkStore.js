class ChunkStore {
    constructor() {
        this.chunks = {};
    }

    addChunk(chunk) {
        this.chunks[chunkKey(chunk.coords)] = chunk;
    }

    getChunk(worldCoords) {
        return this.chunks[chunkKey(worldCoords)];
    }

    updateTile(worldCoords, tileId) {
        const chunk = this.getChunk(worldCoords);
        if (chunk) {
            chunk.updateTile(worldCoords, tileId);
        }
    }
}