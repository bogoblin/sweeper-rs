import init, {decompress} from "./pkg/client.js";

export class MineSocket {
    constructor(url) {
        this.url = url;
        this.connect().then(() => {});
    }

    async connect() {
        await init();

        this.socket = io(this.url);
        this.players = new ClientPlayers();
        this.tileMap = new TileMap();
        this.tileView = new TileView(tileSize, this.tileMap, this.players);

        this.tileMap.socket = this;
        this.tileView.socket = this;

        this.socket.on('connect', () => {
            this.socket.io.engine.on('packet', ({ type, data }) => {
                try {
                    const events = JSON.parse(decompress(new Uint8Array(data)));
                    if (events['Clicked']) {
                        const {player_id, at, updated} = events['Clicked'];
                        const isDead = this.tileMap.chunks.updateRect(updated["top_left"], updated["updated"]);
                        const player = this.players.getPlayer(player_id);
                        player.lastClick = at;
                        if (isDead) {
                            player.kill();
                        }
                    }
                    if (events['Flag']) {
                        const {player_id, at} = events['Flag'];
                        const tile = this.tileMap.chunks.getTile(at);
                        this.tileMap.chunks.updateTile(at, withFlag(tile));
                        this.players.getPlayer(player_id).lastClick = at;
                    }
                    if (events['Unflag']) {
                        const {player_id, at} = events['Unflag'];
                        const tile = this.tileMap.chunks.getTile(at);
                        this.tileMap.chunks.updateTile(at, withoutFlag(tile));
                        this.players.getPlayer(player_id).lastClick = at;
                    }
                } catch (e) {
                }
            });
            this.socket.on('join', ({ player_id }) => {
                this.players.setMyUsername(player_id);
            })
            this.socket.on('chunk', chunk => {
                if (!chunk) {
                    return;
                }
                this.tileMap.chunks.addChunk(new Chunk(chunk.coords, new Uint8Array(chunk.tiles)));
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