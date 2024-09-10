class Player {
    username;
    position;

    constructor(username) {
        this.username = username;
        this.position = [0,0];
        this.lastClick = null;
        this.score = [];
        for (let i=0; i<=8; i++) {
            this.score.push(0);
        }

        this.deadUntil = 0;
    }
    
    kill() {
        this.deadUntil = Date.now() + 10*1000; // TODO: not a magic number
    }

    move(newPosition) {
        this.position = newPosition;
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