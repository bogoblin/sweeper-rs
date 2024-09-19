import {ChunkStore} from "./ChunkStore.js";
import {chunkCoords, chunkSize, defaultChunk} from "./Chunk.js";
import {drawChunkCanvas} from "./TileGraphics.js";
export class TileMap {
    constructor() {
        this.chunks = new ChunkStore();

        // To be set externally to a {MineSocket}
        this.socket = undefined;

        this.lastClicked = 0;

        this.backgroundTileSize = undefined;
        this.backgroundWindowSize = [0,0];
        this.background = document.createElement('canvas');
    }

    draw(
        topLeftWorldCoords,
        bottomRightWorldCoords,
        context,
        tileSize,
        tileView
    ) {
        const [width, height] = [tileView.canvas.width, tileView.canvas.height];

        const firstChunkCoords = chunkCoords(topLeftWorldCoords);
        const lastChunkCoords = chunkCoords(bottomRightWorldCoords);

        // We only need to generate the background once per zoom level
        // It also needs to reset on window resize
        // We can then nudge it around so that it matches up with the chunks
        this.generateBackground(tileSize, [width, height]);
        try {
            const x_off = topLeftWorldCoords[0] - Math.floor(topLeftWorldCoords[0]);
            const y_off = topLeftWorldCoords[1] - Math.floor(topLeftWorldCoords[1]);
            context.drawImage(this.background, -x_off*tileSize, -y_off*tileSize);
        } catch (e) {

        }

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

    generateBackground(tileSize, [width, height]) {
        if (
            this.backgroundWindowSize[0] !== width || this.backgroundWindowSize[1] !== height
        ) {
            this.backgroundTileSize = undefined;
            this.backgroundWindowSize = [width, height];
        }

        if (this.backgroundTileSize === tileSize) {
            return;
        }
        this.backgroundTileSize = tileSize;

        width = width+tileSize;
        height = height+tileSize;
        this.background.width = width;
        this.background.height = height;

        const ctx = this.background.getContext('2d');

        if (tileSize <= 1) {
            ctx.fillStyle = "#bebebe";
            ctx.fillRect(0, 0, width, height);
            return;
        }

        // Instead of drawing every chunk, we draw four, then double it until it fills the background
        // could improve on this but IDK if I care
        drawChunkCanvas(defaultChunk);
        ctx.drawImage(defaultChunk.canvas, 0, 0, tileSize*chunkSize, tileSize*chunkSize);
        let drawnTo = tileSize*chunkSize;
        while (drawnTo < Math.max(width, height)) {
            ctx.drawImage(this.background, 0, 0, drawnTo, drawnTo, drawnTo, 0, drawnTo, drawnTo);
            ctx.drawImage(this.background, 0, 0, drawnTo, drawnTo, 0, drawnTo, drawnTo, drawnTo);
            ctx.drawImage(this.background, 0, 0, drawnTo, drawnTo, drawnTo, drawnTo, drawnTo, drawnTo);
            drawnTo *= 2;
        }
    }
}