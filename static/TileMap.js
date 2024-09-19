import {ChunkStore} from "./ChunkStore.js";
import {chunkCoords, chunkSize, defaultChunk} from "./Chunk.js";
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

        // Iterate through the chunks and draw them
        for (let chunkY=firstChunkCoords[1]; chunkY<=lastChunkCoords[1]; chunkY+=chunkSize) {
            for (let chunkX=firstChunkCoords[0]; chunkX<=lastChunkCoords[0]; chunkX+=chunkSize) {
                const chunk = this.chunks.getChunk([chunkX, chunkY]) || defaultChunk;
                const screenCoords = tileView.worldToScreen([chunkX, chunkY]);
                drawChunkCanvas(chunk);
                context.drawImage(chunk.canvas, ...screenCoords, tileSize*chunkSize, tileSize*chunkSize);
            }
        }
    }

    doubleClickTime = 100; // milliseconds
    click(worldCoords) {
        if (!this.socket) { return }
        const now = performance.now();
        if (now - this.lastClicked < this.doubleClickTime) {
            this.doubleClick(worldCoords);
        }
        else {
            this.socket.sendClickMessage(worldCoords);
        }
        this.lastClicked = now;
    }

    rightClick(worldCoords) {
        if (!this.socket) { return }
        this.socket.sendFlagMessage(worldCoords);
    }

    doubleClick(worldCoords) {
        if (!this.socket) { return }
        this.socket.sendDoubleClickMessage(worldCoords);
    }

}