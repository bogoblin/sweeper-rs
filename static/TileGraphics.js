const tileSize = 16;

const sprites = new Image();
sprites.src ='./tiles.png';

let debug = false;

const getSpriteIndex = (tile) => {
    if (!revealed(tile)) {
        if (flag(tile)) return 10;
        else return 9;
    }
    if (mine(tile)) return 11;
    return adjacent(tile);
}

/**
 *
 * @param context {CanvasRenderingContext2D}
 * @param canvasCoords {number[]}
 * @param tile {number}
 */
const drawTileToCanvasContext = (context, [x, y], tile) => {
    const spriteIndex = getSpriteIndex(tile);
    context.drawImage(sprites, // source image
        spriteIndex * tileSize, 0, tileSize, tileSize, // left, top, width and height of rectangle in source image
        x, y, tileSize, tileSize // left, top, width and height of rectangle on the canvas
    );
}

const drawChunkCanvas = (chunk) => {
    if (!chunk.redraw) {
        return;
    }
    if (!chunk.canvas) {
        chunk.canvas = document.createElement('canvas');
    }
    chunk.canvas.width = tileSize * chunkSize;
    chunk.canvas.height = tileSize * chunkSize;

    if (spritesAreLoaded()) {
        const chunkCtx = chunk.canvas.getContext('2d');
        let index = 0;
        const rect = [[0,0], [chunkSize, chunkSize]];
        forEachInRect(rect, (tileCoords) => {
            const tile = chunk.tiles[index];
            const canvasCoords = vectorTimesScalar(tileCoords, tileSize);
            drawTileToCanvasContext(chunkCtx, canvasCoords, tile);

            index += 1;
        });

        chunk.redraw = false;
    } else {
        addLoadCallbackForSprites(() => chunk.redraw = true);
    }
}

const spritesAreLoaded = () => sprites.complete;

const addLoadCallbackForSprites = (callback) => {
    sprites.addEventListener('load', callback);
}