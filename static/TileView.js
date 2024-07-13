class TileView {
    /**
     * @param tileSize {number}
     * @param tileMap {TileMap}
     * @param players {ClientPlayers}
     */
    constructor( tileSize, tileMap, players ) {
        this.tileSize = tileSize;
        this.tileMap = tileMap;
        this.players = players;

        this.viewCenter = [0,0];
        this.mouseCoords = [0,0];

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
    }

    setCanvas(newCanvas) {
        if (!newCanvas) {
            return;
        }
        this.canvas = newCanvas;
        this.context = this.canvas.getContext('2d');

        this.buttonsDown = [false,false,false]; // left, middle, right

        this.canvas.addEventListener('mousedown', this.mouseDown.bind(this));
        this.canvas.addEventListener('mouseup', this.mouseUp.bind(this));
        this.canvas.addEventListener('mousemove', this.mouseMove.bind(this));

        this.canvas.oncontextmenu = () => false; // disable right click

        this.updateCanvasSize();
        this.draw();
    }

    mouseDown(event) {
        const button = event.button;
        this.buttonsDown[button] = true;
        const screenCoords = this.getScreenCoords(event);

        switch (button) {
            case 0:
            case 1:
                this.drag.dragStartScreen = screenCoords;
                this.drag.viewCenterOnDragStart = this.viewCenter;
                break;
            case 2:
                const worldCoords = this.screenToWorld(screenCoords);
                this.tileMap.rightClick(worldCoords);
                break;
        }
    }

    mouseUp(event) {
        const button = event.button;
        this.buttonsDown[button] = false;
        const screenCoords = this.getScreenCoords(event);

        if (button === 0 || button === 1) {
            const dragVector = vectorSub(this.drag.dragStartScreen, screenCoords);
            if (vectorMagnitudeSquared(dragVector) >= 3) {
                if (this.socket) {
                    this.socket.sendMoveMessage(this.viewCenter);
                }
                return;
            }
        }

        const worldCoords = this.screenToWorld(screenCoords);
        switch (button) {
            case 0: // left click
                if (this.buttonsDown[2]) {
                    this.tileMap.doubleClick(worldCoords);
                }
                else if (event.shiftKey) {
                    this.tileMap.rightClick(worldCoords);
                }
                else {
                    this.tileMap.click(worldCoords);
                }
                break;
            default:
        }
    }

    mouseMove(event) {
        const screenCoords = this.getScreenCoords(event);
        this.mouseCoords = this.screenToWorldInt(screenCoords);
        if (this.buttonsDown[0] || this.buttonsDown[1]) {
            const dragVector = vectorSub(this.drag.dragStartScreen, screenCoords);
            const dragVectorInWorldSpace = vectorTimesScalar(dragVector, 1 / this.tileSize);
            this.viewCenter = vectorAdd(this.drag.viewCenterOnDragStart, dragVectorInWorldSpace);
        }
    }

    updateCanvasSize() {
        this.canvas.width = window.innerWidth;
        this.canvas.height = window.innerHeight;
    }

    getScreenCoords(event) {
        const {left, top} = this.canvas.getBoundingClientRect();
        const screenCoords = [event.clientX, event.clientY];

        return vectorSub(screenCoords, [left, top]);
    }

    draw() {
        const { width, height } = this.canvas;
        const topLeftWorldCoords = this.screenToWorldInt([0,0]);
        const bottomRightWorldCoords = this.screenToWorldInt([width, height]);
        this.tileMap.draw(
            topLeftWorldCoords,
            bottomRightWorldCoords,
            this.context,
            this.tileSize,
            this
        );

        this.players.draw(this);

        // Debug
        drawText(this.context, `View Centre: ${this.viewCenter}`, [10,10]);
        drawText(this.context, `Mouse Position: ${this.mouseCoords}`, [10,40]);

        requestAnimationFrame(() => {
            this.draw();
        });
    }

    screenToWorld(screenCoords) {
        const { width, height } = this.canvas;
        const ts = this.tileSize;
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
        const ts = this.tileSize;
        const screenCenter = [width/2, height/2];

        // Calculate the vector that goes from the world position to the world center
        const worldToCenter = vectorSub(this.viewCenter, worldCoords);

        // Convert this into screen space
        const distanceFromViewCenterInScreenSpace = vectorTimesScalar(worldToCenter, ts);

        // Subtract from the screen center to get result
        return vectorSub(screenCenter, distanceFromViewCenterInScreenSpace);
    }
}