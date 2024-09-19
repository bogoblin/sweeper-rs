import {ChunkStore} from "./ChunkStore.js";
import {chunkCoords, chunkSize} from "./Chunk.js";
import {drawChunkCanvas} from "./TileGraphics.js";
export class TileMap {
    constructor() {
        this.chunks = new ChunkStore();

        // To be set externally to a {MineSocket}
        this.socket = undefined;

        this.lastClicked = 0;
    }

    draw(
        topLeftWorldCoords,
        bottomRightWorldCoords,
        context,
        tileSize,
        tileView
    ) {
        const firstChunkCoords = chunkCoords(topLeftWorldCoords);
        const lastChunkCoords = chunkCoords(bottomRightWorldCoords);

        // If the last chunk is before the first chunk then we will get stuck in an infinite loop -
        // this shouldn't happen but let's prevent against it
        if (firstChunkCoords[0] > lastChunkCoords[0] || firstChunkCoords[1] > lastChunkCoords[1]) {
            return;
        }

        context.fillRect(0, 0, 5000, 5000);

        const chunkKeys = this.chunks.queryChunks(firstChunkCoords, lastChunkCoords);
        // Iterate through the chunks and draw them
        for (const key of chunkKeys) {
            const chunk = this.chunks.chunks[key];
            if (!chunk) {
                console.log("Why no chunk???");
                continue;
            }
            const screenCoords = tileView.worldToScreen(chunk.coords);
            drawChunkCanvas(chunk);
            context.drawImage(chunk.canvas, ...screenCoords, tileSize*chunkSize, tileSize*chunkSize);
        }
    }
}