class Player {
    username;
    socket;
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
        this.deaths = 0;
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