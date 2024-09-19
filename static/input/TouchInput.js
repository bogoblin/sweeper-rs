export class TouchInput {
    constructor(tileView) {
        this.tileView = tileView;
        this.canvas = tileView.canvas;

        this.canvas.addEventListener('touchstart', this.touchStart.bind(this));
        this.canvas.addEventListener('touchend', this.touchEnd.bind(this));
        this.canvas.addEventListener('touchmove', this.touchMove.bind(this));

        this.modeChangeForm = document.getElementById('touchMode');
        this.mode = "flag";
        for (const e of this.modeChangeForm.elements) {
            e.addEventListener('input', () => {
                if (e.checked && e.name === 'touchMode') {
                    this.mode = e.value;
                }
            });
            if (e.checked && e.name === 'touchMode') {
                this.mode = e.value;
            }
        }

        this.ongoingTouches = [];
    }

    touchStart(event) {
        event.preventDefault();
        const touches = event.changedTouches;

        for (let i = 0; i < touches.length; i++) {
            this.ongoingTouches.push(touches[i]);
            let screenCoords = [touches[i].pageX, touches[i].pageY];
            this.tileView.startDrag(screenCoords);
        }
    }

    touchEnd(event) {
        event.preventDefault();
        const touches = event.changedTouches;

        for (let i = 0; i < touches.length; i++) {
            this.ongoingTouches = this.ongoingTouches.filter(touch =>
                touch.identifier !== touches[i].identifier
            );
            let screenCoords = [touches[i].pageX, touches[i].pageY];
            const wasADrag = this.tileView.endDrag(screenCoords);
            if (wasADrag) {
                return;
            }

            if (this.tileView.isRevealedAt(screenCoords)) {
                this.tileView.doubleClickAt(screenCoords);
            } else {
                if (this.mode === 'flag') {
                    this.tileView.flagAt(screenCoords);
                } else {
                    this.tileView.clickAt(screenCoords);
                }
            }
        }
    }

    touchMove(event) {
        event.preventDefault();
        const touches = event.changedTouches;

        for (let i = 0; i < touches.length; i++) {
            let screenCoords = [touches[i].pageX, touches[i].pageY];
            this.tileView.updateDrag(screenCoords);
        }
    }
}