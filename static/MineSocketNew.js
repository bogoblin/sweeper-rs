
class MineSocket {
    onError = () => {
    };
    onWelcome = () => {
    };
    onLogout = () => {
    };
    onPlayerUpdate = () => {
    };

    currentError;

    constructor(url) {
        this.url = url;
        this.reset();
    }

    reset() {
        this.socket = new WebSocket(this.url);
        this.players = new ClientPlayers();
        this.tileMap = new TileMap();
        this.tileView = new TileView(tileSize, this.tileMap, this.players);

        this.tileMap.socket = this;
        this.tileView.socket = this;

        this.socket.addEventListener("open", (event) => {

        });
        this.socket.addEventListener("message", (event) => {
            const message = deserialize(event.data);

            switch (message.type) {
                case 'chunk': {
                    const chunk = message.chunk;
                    if (!chunk) {
                        return;
                    }
                    this.tileMap.chunks.addChunk(new Chunk(chunk.coords, new Uint8Array(chunk.tiles)));
                }
                    break;
                case 'updated_rect': {
                    const {topLeft, updated} = message;
                    let num_updated = 0;
                    for (let relative_x = 0; relative_x < updated.length; relative_x++) {
                        for (let relative_y = 0; relative_y < updated[0].length; relative_y++) {
                            let x = relative_x + topLeft[0];
                            let y = relative_y + topLeft[1];
                            const updatedTile = updated[relative_x][relative_y];
                            if (updatedTile !== 0) {
                                this.tileMap.chunks.updateTile([x, y], updatedTile);
                                num_updated += 1;
                            }
                        }
                    }
                }
                    break;
                case 'flag': {
                    const {position} = message;
                    const tile = this.tileMap.chunks.getTile(position);
                    this.tileMap.chunks.updateTile(position, withFlag(tile));
                }
                    break;
                case 'unflag': {
                    const {position} = message;
                    const tile = this.tileMap.chunks.getTile(position);
                    this.tileMap.chunks.updateTile(position, withoutFlag(tile));
                }
                    break;
                case 'player': {
                    const {player} = message;
                    if (!player) {
                        return;
                    }
                    this.players.updatePlayer(player);
                    this.onPlayerUpdate();
                }
                    break;
                case 'welcome': {
                    const {username} = message;
                    this.players.setMyUsername(username);
                    this.onWelcome();
                    this.onPlayerUpdate();
                    this.tileView.viewCenter = this.players.me().position;
                }
                    break;
            }
        })
    }

    sendClickMessage(coords) { // c for click
        const message = ['click', ...coords];
        this.socket.send(JSON.stringify(message));
    }

    sendFlagMessage(coords) { // f for flag
        const message = ['flag', ...coords];
        this.socket.send(JSON.stringify(message));
    }

    sendDoubleClickMessage(coords) { // d for double click
        const message = ['doubleClick', ...coords];
        this.socket.send(JSON.stringify(message));
    }

    sendMoveMessage(coords) { // m for move
        const message = ['move', ...coords];
        this.socket.send(JSON.stringify(message));
    }
}