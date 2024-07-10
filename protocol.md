# The Onlinesweeper Protocol

## Events

Events are the way that the server communicates updates to the client.

The server will send numbered messages regarding events:

...
1000 Player registered
1001 Player clicked and updated 1 tile at (10, 20)
1002 Player clicked and updated a bunch of tiles: ...
1003 Player added a flag at (11, 25)
...

The client doesn't need to acknowledge these messages. Instead, if a message is missing, the client can ask for it to be repeated.


## Queries

The client can ask the server for information on Chunks and Players. The response will contain the
latest message index. 
