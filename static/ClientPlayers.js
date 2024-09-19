import {Player} from "./Player.js";
import {vectorAdd, vectorTimesScalar} from "./Vector2.js";

const cursor = new Image();
cursor.src = "cursor.png";

export class ClientPlayers {
    constructor() {
        this.players = {};
    }

    /**
     * Get the player with the given username
     * @param username
     * @returns {Player}
     */
    getPlayer(username) {
        const existingPlayer = this.players[username];
        if (existingPlayer) {
        } else {
            this.players[username] = new Player(username);
        }
        return this.players[username];
    }

    /**
     * @returns {null|Player}
     */
    me() {
        if (!this.myUsername) {
            return null;
        }
        return this.getPlayer(this.myUsername);
    }

    setMyUsername(username) {
        this.myUsername = username;
    }


    /**
     *
     * @param tileView {TileView}
     */
    draw(tileView) {
        const context = tileView.context;
        const {width, height} = context.canvas;

        // draw cursors
        for (let player of Object.values(this.players)) {
            if (player === this.me()) {
                continue;
            }
            if (!player.lastClick) {
                continue;
            }
            player.cursorTarget = vectorAdd(player.lastClick, [0.5, 0.5]);
            if (!player.cursorDisplay) {
                player.cursorDisplay = player.cursorTarget;
            }
            player.cursorDisplay = (vectorTimesScalar(vectorAdd(player.cursorDisplay, player.cursorTarget), 1/2));
            const [x, y] = tileView.worldToScreen(player.cursorDisplay);

            // TODO: different cursors for different players
            context.drawImage(cursor, x, y);
        }

        // if you are dead, make the screen red and show the respawn time
        if (this.me() && !this.me().isAlive()) {
            const secondsUntilRespawn = this.me().timeUntilRespawn() / 1000;
            const deathTextHeight = Math.max(width * 0.1, 200);
            context.font = `${deathTextHeight}px monospace`;
            context.fillStyle = 'black';
            context.textBaseline = 'middle';
            context.textAlign = 'center';
            context.fillText(secondsUntilRespawn.toFixed(1), width/2, height/2);

            context.font = `${deathTextHeight * 0.2}px monospace`;
            context.fillText('You are dead. Respawning...', width/2, height/2 - deathTextHeight/2 - deathTextHeight*0.05);

            const deathOverlayOpacity = Math.min(0.3, secondsUntilRespawn*0.3);
            context.fillStyle = `rgba(255, 0, 0, ${deathOverlayOpacity})`;
            context.fillRect(0, 0, width, height);
        }
    }
}