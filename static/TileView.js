import {vectorAdd, vectorMagnitudeSquared, vectorSub, vectorTimesScalar} from "./Vector2.js";
import {MouseInput} from "./input/MouseInput.js";
import {TouchInput} from "./input/TouchInput.js";
import {revealed} from "./Tile.js";

export class TileView {
    /**
     * @param tileSize {number}
     * @param tileMap {TileMap}
     * @param players {ClientPlayers}
     */
    constructor( tileSize, tileMap, players ) {
        this.tileSize = tileSize;
        this.tileMap = tileMap;
        this.players = players;

        this.url = new URL(window.location.href);
        const view_x = this.getPositionParam('x');
        const view_y = this.getPositionParam('y');

        this.viewCenter = [view_x, view_y];

        this.setCanvas(document.getElementById("gameCanvas"));

        window.addEventListener("resize", () => {
            this.updateCanvasSize();
        });

        this.drag = {
            dragStartScreen: [0,0],
            viewCenterOnDragStart: [0,0]
        }

        // Set by MineSocket
        this.socket = undefined;

        const isOnMobile = navigator.userAgent.includes('Android') || navigator.userAgent.includes('iPhone');
        this.zoomLevel = isOnMobile ? 16 : 0;
    }

    getPositionParam(key) {
        let param = parseFloat(this.url.searchParams.get(key)) || 0;
        return param;
    }

    zoomFactor = Math.sqrt(Math.sqrt(Math.sqrt(2)));
    getScreenTileSize() {
        return this.tileSize * Math.pow(this.zoomFactor, this.zoomLevel);
    }

    setCanvas(newCanvas) {
        if (!newCanvas) {
            return;
        }
        this.canvas = newCanvas;
        this.context = this.canvas.getContext('2d');

        this.mouseInput = new MouseInput(this);
        this.touchInput = new TouchInput(this);

        this.updateCanvasSize();
        this.draw();
    }


    updateCanvasSize() {
        this.canvas.width = window.innerWidth;
        this.canvas.height = window.innerHeight;
    }

    flagAt(screenCoords) {
        const worldCoords = this.screenToWorldInt(screenCoords);
        this.socket?.sendFlagMessage(worldCoords);
    }
    clickAt(screenCoords) {
        const worldCoords = this.screenToWorldInt(screenCoords);
        this.socket?.sendClickMessage(worldCoords);
    }
    doubleClickAt(screenCoords) {
        if (this.isRevealedAt(screenCoords)) {
            const worldCoords = this.screenToWorldInt(screenCoords);
            this.socket?.sendDoubleClickMessage(worldCoords);
        } else {
            this.clickAt(screenCoords);
        }
    }

    isRevealedAt(screenCoords) {
        const worldCoords = this.screenToWorldInt(screenCoords);
        return revealed(this.socket.tileMap.chunks.getTile(worldCoords));
    }

    startDrag(screenCoords) {
        this.drag.dragStartScreen = screenCoords;
        this.drag.viewCenterOnDragStart = this.viewCenter;
    }
    updateDrag(screenCoords) {
        const dragVector = vectorSub(this.drag.dragStartScreen, screenCoords);
        const dragVectorInWorldSpace = vectorTimesScalar(dragVector, 1 / this.getScreenTileSize());
        this.viewCenter = vectorAdd(this.drag.viewCenterOnDragStart, dragVectorInWorldSpace);
    }
    endDrag(screenCoords) {
        const dragVector = vectorSub(this.drag.dragStartScreen, screenCoords);
        if (vectorMagnitudeSquared(dragVector) >= 3) {
            if (this.socket) {
                this.socket.sendMoveMessage(this.viewCenter);
            }
            const [x, y] = this.viewCenter;
            this.url.searchParams.set('x', x.toString());
            this.url.searchParams.set('y', y.toString());
            window.history.replaceState(null, '', this.url.toString());
            return true;
        }
        return false;
    }

    zoomIn(amount, screenCoords) {
        const mousePositionBeforeZoom = this.screenToWorld(screenCoords);
        if (amount < 0) {
            this.zoomLevel += 1;
        } else {
            this.zoomLevel -= 1;
        }
        const mousePositionAfterZoom = this.screenToWorld(screenCoords);
        const difference = vectorSub(mousePositionBeforeZoom, mousePositionAfterZoom);
        this.viewCenter = vectorAdd(this.viewCenter, difference);
    }

    draw() {
        const { width, height } = this.canvas;
        const topLeftWorldCoords = this.screenToWorldInt([0,0]);
        const bottomRightWorldCoords = this.screenToWorldInt([width, height]);
        this.tileMap.draw(
            topLeftWorldCoords,
            bottomRightWorldCoords,
            this.context,
            this.getScreenTileSize(),
            this
        );

        this.players.draw(this);

        // Debug
        // drawText(this.context, `View Centre: ${this.viewCenter}`, [10,10]);
        // drawText(this.context, `Mouse Position: ${this.mouseCoords}`, [10,40]);

        requestAnimationFrame(() => {
            this.draw();
        });
    }

    screenToWorld(screenCoords) {
        const { width, height } = this.canvas;
        const ts = this.getScreenTileSize();
        const screenCenter = [width/2, height/2];

        // Calculate the vector that goes from the screen position to the center of the screen
        const screenToCenter = vectorSub(screenCenter, screenCoords);

        // Convert this into world space
        const distanceFromViewCenterInWorldSpace = vectorTimesScalar(screenToCenter, 1/ts);

        // Subtract from the view center to get result
        return vectorSub(this.viewCenter, distanceFromViewCenterInWorldSpace);
    }

    screenToWorldInt(screenCoords) {
        return this.screenToWorld(screenCoords).map((v) => Math.floor(v));
    }

    worldToScreen(worldCoords) {
        const { width, height } = this.canvas;
        const ts = this.getScreenTileSize();
        const screenCenter = [width/2, height/2];

        // Calculate the vector that goes from the world position to the world center
        const worldToCenter = vectorSub(this.viewCenter, worldCoords);

        // Convert this into screen space
        const distanceFromViewCenterInScreenSpace = vectorTimesScalar(worldToCenter, ts);

        // Subtract from the screen center to get result
        return vectorSub(screenCenter, distanceFromViewCenterInScreenSpace);
    }
}