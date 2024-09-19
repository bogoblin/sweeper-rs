import {vectorSub} from "../Vector2.js";

const LEFT = 0;
const MIDDLE = 1;
const RIGHT = 2;

export class MouseInput {
    constructor(tileView) {
        this.tileView = tileView;
        this.canvas = tileView.canvas;

        this.buttonsDown = [false,false,false]; // left, middle, right
        this.mousePosition = [0, 0];

        this.canvas.addEventListener('mousedown', this.mouseDown.bind(this));
        this.canvas.addEventListener('mouseup', this.mouseUp.bind(this));
        this.canvas.addEventListener('mousemove', this.mouseMove.bind(this));
        this.canvas.addEventListener('wheel', this.mouseWheel.bind(this));

        this.canvas.oncontextmenu = () => false; // disable right click
    }

    getScreenCoords(event) {
        const {left, top} = this.canvas.getBoundingClientRect();
        const screenCoords = [event.clientX, event.clientY];

        return vectorSub(screenCoords, [left, top]);
    }

    mouseDown(event) {
        const button = event.button;
        this.buttonsDown[button] = true;
        const screenCoords = this.getScreenCoords(event);

        switch (button) {
            case LEFT:
            case MIDDLE:
                this.tileView.startDrag(screenCoords);
                break;
            case RIGHT:
                this.tileView.flagAt(screenCoords);
                break;
        }
    }

    mouseUp(event) {
        const button = event.button;
        this.buttonsDown[button] = false;
        const screenCoords = this.getScreenCoords(event);

        if (button === LEFT || button === MIDDLE) {
            const wasADrag = this.tileView.endDrag(screenCoords);
            // If we were dragging, then don't send a click.
            if (wasADrag) {
                return;
            }
        }

        switch (button) {
            case LEFT:
                if (this.buttonsDown[RIGHT]) {
                    this.tileView.doubleClickAt(screenCoords);
                }
                else if (event.shiftKey) {
                    this.tileView.flagAt(screenCoords);
                }
                else {
                    this.tileView.clickAt(screenCoords);
                }
                break;
            default:
        }
    }

    mouseMove(event) {
        const screenCoords = this.getScreenCoords(event);
        if (this.buttonsDown[LEFT] || this.buttonsDown[MIDDLE]) {
            this.tileView.updateDrag(screenCoords);
        }
        this.mousePosition = screenCoords;
    }

    mouseWheel(event) {
        this.tileView.zoomIn(event.deltaY, this.mousePosition);
    }
}