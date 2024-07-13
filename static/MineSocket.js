class MineSocket {
    onError = () => {};
    onWelcome = () => {};
    onPlayerUpdate = () => {};

    currentError;

    constructor(url) {
        this.url = url;
        this.connect();

    }

    signUp(username) {
        console.log(`register ${username}`);
        this.socket.emit('message', [ 'register', username ]);
    }

    logIn(authKey) {
        setTimeout(() => {
            console.log(`login ${authKey}`);
            this.socket.emit('message', [ 'login', authKey ]);
        }, 2000);
    }

    showLoginBox() {
        const loginDialogElement = document.getElementById("login");
        loginDialogElement.setAttribute("open", "open");
        loginDialogElement.getElementsByTagName("form")[0].onsubmit = () => {
            const username = document.getElementById("nameEntry").value;
            loginDialogElement.removeAttribute("open");
            this.signUp(username);
        }
    }

    connect() {
        this.socket = io(this.url);
        this.players = new ClientPlayers();
        this.tileMap = new TileMap();
        this.tileView = new TileView(tileSize, this.tileMap, this.players);

        this.tileMap.socket = this;
        this.tileView.socket = this;

        this.socket.on('connect', () => {
            this.socket.on('login_details', ({username, authKey}) => {
                localStorage.setItem("authKey", authKey);
                this.logIn(authKey);
            });
            this.socket.on('chunk', chunk => {
                if (!chunk) {
                    return;
                }
                this.tileMap.chunks.addChunk(new Chunk(chunk.coords, new Uint8Array(chunk.tiles)));
            });
            this.socket.on('updated_rect', ({ topLeft, updated }) => {
                this.tileMap.chunks.updateRect(topLeft, updated);
            });
            this.socket.on('flag', ({ position }) => {
                const tile = this.tileMap.chunks.getTile(position);
                this.tileMap.chunks.updateTile(position, withFlag(tile));
            });
            this.socket.on('unflag', ({ position }) => {
                const tile = this.tileMap.chunks.getTile(position);
                this.tileMap.chunks.updateTile(position, withoutFlag(tile));
            });
            this.socket.on('player', player => {
                if (!player) {
                    return;
                }
                this.players.updatePlayer(player);
                this.onPlayerUpdate();
            });
            this.socket.on('leave', username => {
                this.players.removePlayer(username);
            })
            this.socket.on('welcome', username => {
                this.players.setMyUsername(username);
                this.onWelcome();
                this.onPlayerUpdate();
                this.tileView.viewCenter = this.players.me().position;
            });
            this.socket.on('error', error => {
                console.log(error);
                this.error(error);
                if (error["error"] === "Auth key not recognised") {
                    localStorage.removeItem("authKey");
                    this.showLoginBox();
                }
            });

            const authKey = localStorage.getItem("authKey");
            if (!authKey) {
                this.showLoginBox();
            } else {
                console.log("going to send the auth key");
                this.logIn(authKey);
            }
        });
    }

    sendClickMessage(coords) { // c for click
        this.socket.emit('message', ['click', ...coords]);
    }

    sendFlagMessage(coords) { // f for flag
        this.socket.emit('message', ['flag', ...coords]);
    }

    sendDoubleClickMessage(coords) { // d for double click
        this.socket.emit('message', ['doubleClick', ...coords]);
    }

    sendMoveMessage(coords) { // m for move
        this.socket.emit('message', ['move', ...coords]);
    }

    error(err) {
        this.currentError = err;
        if (this.onError) {
            this.onError();
        }
    }
}