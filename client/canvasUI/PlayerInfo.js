export class PlayerInfo {
    constructor(player) {
        this.player = player;
    }

    /**
     * Draw this to the canvas context
     * @param context {CanvasRenderingContext2D}
     * @param canvasX {number}
     * @param canvasY {number}
     */
    draw(context, [canvasX, canvasY]) {
        context.fillStyle = '#FF00FF';
        context.font = 'sans-serif';
        context.fillText(this.player.username, canvasX, canvasY);
        context.fillText(this.player.points(), canvasX, canvasY + 50);
    }
}