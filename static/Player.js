class Player {
    username;
    socket;
    hashedPassword;
    position;

    constructor(username, hashedPassword) {
        this.username = username;
        this.hashedPassword = hashedPassword;
        this.position = [0,0];
        this.lastClick = null;
        this.score = [];
        for (let i=0; i<=8; i++) {
            this.score.push(0);
        }

        this.deadUntil = 0;
        this.deaths = 0;
    }

    publicVersion() {
        return {
            username: this.username,
            position: this.position,
            lastClick: this.lastClick,
            score: this.score,
            deadUntil: this.deadUntil,
            deaths: this.deaths,
        };
    }

    connect(socket) {
        this.socket = socket;
        socket.player = this;
        const session = socket.request.session;
        session.username = this.username;
        session.save();
    }

    move(newPosition) {
        this.position = newPosition;
    }

    /**
     * Kill this player
     * @param deathDuration {number} time until respawn in milliseconds
     */
    kill(deathDuration) {
        this.deadUntil = Date.now() + deathDuration;
        this.deaths++;
    }

    hasRevealed(tile, world) {
        const info = tileInfo(tile);

        if (info.mine) {
            world.killPlayer(this, 5000 * this.deaths);
        }
        else {
            this.score[info.adjacent] += 1;
        }
    }

    isAlive() {
        return Date.now() > this.deadUntil;
    }

    timeUntilRespawn() {
        if (this.isAlive()) {
            return 0;
        }
        return this.deadUntil - Date.now();
    }

    points() {
        let pointTotal = 0;
        for (let i=0; i<=8; i++) {
            pointTotal += this.score[i] * Math.pow(i, 4);
        }
        return pointTotal;
    }
}