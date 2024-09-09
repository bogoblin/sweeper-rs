class MineSocket {
    constructor(url) {
        this.url = url;
        this.connect();
    }

    connect() {
        this.socket = io(this.url);
        this.players = new ClientPlayers();
        this.tileMap = new TileMap();
        this.tileView = new TileView(tileSize, this.tileMap, this.players);

        this.tileMap.socket = this;
        this.tileView.socket = this;

        this.socket.on('connect', () => {
            this.socket.on('join', ({ player_id }) => {
                this.players.setMyUsername(player_id);
            })
            this.socket.on('chunk', chunk => {
                if (!chunk) {
                    return;
                }
                this.tileMap.chunks.addChunk(new Chunk(chunk.coords, new Uint8Array(chunk.tiles)));
            });
            this.socket.on('click', ({ position, player_id, updated_rect }) => {
                const isDead = this.tileMap.chunks.updateRect(updated_rect["top_left"], updated_rect["updated"]);
                const player = this.players.getPlayer(player_id);
                player.lastClick = position;
                if (isDead) {
                    player.kill();
                }
            });
            this.socket.on('flag', ({ position, player_id }) => {
                const tile = this.tileMap.chunks.getTile(position);
                this.tileMap.chunks.updateTile(position, withFlag(tile));
                this.players.getPlayer(player_id).lastClick = position;
            });
            this.socket.on('unflag', ({ position, player_id }) => {
                const tile = this.tileMap.chunks.getTile(position);
                this.tileMap.chunks.updateTile(position, withoutFlag(tile));
                this.players.getPlayer(player_id).lastClick = position;
            });
        });
    }

    sendClickMessage(coords) { // c for click
        if (this.players.me().isAlive())
            this.socket.emit('message', ['click', ...coords]);
    }

    sendFlagMessage(coords) { // f for flag
        if (this.players.me().isAlive())
            this.socket.emit('message', ['flag', ...coords]);
    }

    sendDoubleClickMessage(coords) { // d for double click
        if (this.players.me().isAlive())
            this.socket.emit('message', ['doubleClick', ...coords]);
    }

    sendMoveMessage(coords) { // m for move
        if (this.players.me().isAlive())
            this.socket.emit('message', ['move', ...coords]);
    }
}