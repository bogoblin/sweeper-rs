class TileMap {
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
                const chunk = this.chunks.getChunk([chunkX, chunkY]);
                let chunkToDraw;
                if (chunk) {
                    chunkToDraw = chunk;
                }
                else {
                    chunkToDraw = defaultChunk;
                }
                chunkToDraw.draw(context, tileView.worldToScreen([chunkX, chunkY]), drawChunkCanvas);
            }
        }
    }

    doubleClickTime = 600; // milliseconds
    click(worldCoords) {
        console.log(`Clicked at ${worldCoords}`);
        const now = performance.now();
        if (this.socket) {
            if (now - this.lastClicked < this.doubleClickTime) {
                this.doubleClick(worldCoords);
            }
            else {
                this.socket.sendClickMessage(worldCoords);
            }
        }
        this.lastClicked = now;
    }

    rightClick(worldCoords) {
        if (this.socket) {
            this.socket.sendFlagMessage(worldCoords);
        }
    }

    doubleClick(worldCoords) {
        console.log(`Double Clicked at ${worldCoords}`);
        this.socket.sendDoubleClickMessage(worldCoords);
    }

    addChunk(chunk) {
        this.chunks.addChunk(chunk);
    }
}