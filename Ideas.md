What do I want to have for the packets?
-----------------------------

The packets need to be handled by different structs, but I don't want them all
to be saved in the nethandler, because that makes changing anything very difficult.
My proposed Idea is as follows:

NetHandler     NetClient       Master          GameHandler           Game
    |                |           |                  |
	|                |           |                  |
	|----------------+---------->+----------------->|
	|===============>|           |                  |=================>|
	|                |           |                  |                  |
	|                |------------------------------------------------>|
	|                |           |                  |                  |
	|                |------------------------------------------------>|
	|                |           |                  |                  |
	|                |           |                  |                  |
	|                |<------------------------------------------------|
	|                |           |                  |                  |
	|                |           |                  |<=================|
	|                |           |                  |
	|                |           |                  |
	|<===============|           |                  |
	|                            |                  |

...

The thick lines represent creation and destruction of the respective elements.
The thin lines represent packet flow. Time flows from top to bottom.
In this case:
The NetHandler, Master and GameHandler structures exist completely in parallel,
however the NetHandler will be needed to provide Master and GameHandler with
packets.
Suppose a client connects to the server. The NetHandler registers this, since
it is responsible for listening to incoming connections.
It creates the NetClient, starting it on another thread, which will receive the
packets from the client. Also, the client gets a sender to send all the packets
to.
And already you have a huge clusterf**k no-one but me will ever get through
without 10+ hours of explaining, IF I myself will even understand what the hell
I've been doing.

What if you could steal, abandon or give_away ownership of an object between
threads?
This way, only a single handler could always handle a client. The client should
however also be able to push itself around. In this way, only one thread always
handles the client, but the interpretation should (hopefully) be easy to understand
and it stays flexible.
Also: This might be possible to implement via simple mpsc, but it could be
difficult to make the client able to send itself.

Implementing this would probably run against many things rust stands for and
sounds like a radical idea for a problem that can be solved easily.

Maybe more like a dynamic match might be what we are looking for. Every packet
has to be handled by someone, but only once. This has to be able to be done for
all clients and client-wise optionally though.

(On a sidenote: The idea is nice and all, but for othello there really is no
need to actually have every game be in another thread. I could probably make a
version first, where the clients send all packets to the nethandler, which
distributes them to the master and the gamehandler.)

Next problem in implementing this:
No packet should be left unhandled, but there are multiple threads that want to
access the packets. Any packet must not be handled twice. That means:
When I get the next packet of the NetHandler, I am the only one who gets this
packet. Then I decide if I want to handle it. IF I do, and only if that is the
case, the packet will be removed from the buffer. If I decide against that,
the packet must be left in the buffer. However, if I leave the packet at the top
of the buffer and another thread also wants to handle its packets, it would quite
possible handle the packets in an incorrect order.
At first sight it sounds like a good idea to always try to pop the oldest packet
but leave it, in case it cannot be handled by this particular handler.
However usually one handlers operations are of no concern to the next, and if
a packet is not handled by the handler, it can just ignore it.

CORE: Should it be possible for another handler to process a newer packet?

This could lead to problems like this:
The GameHandler gets a packet that says a game has been cancelled by a client,
but before that the corresponding game should receive a stone setting by the
other client.

The GameHandler would now cancel the game, but the packet of the other client
would still be in the queue. Since the Game doesn't exist anymore, the packet is
still in the queue, but noone will ever handle it, or worse yet, when the players
start a new game, it might again be handled by the game. The GameHandler would be
therefore responsible for the cleanup, or the Game would always have to manually
cleanup after it is dropped, just to make sure.
A maximum lifetime for packets, i.e. garbage collection is not desirable.

The other option might block sometimes, but I think it is more desirable to try
for now.
This way you also notice, when a packet is not handled, because the server blocks
indefinately, drawing attention to a bug.

Well that's a stupid Idea. It just means the whole program is nothing more but
an unneccessarily contrived single-threaded program, since the next thread must
wait until the other thread is finished and then the other threads in turn need
to wait again. Let's just make sure, that everything is cleaned up alright.

The packets can be accessed via a VecDeque which will be copied for everyone
so that they can truly access their packets simultaneously. The network thread
can update the queues.

Still, the distribution of packets is not entirely clear.
Let's just say we have two clients on the server. They are currently *not* engaged
in a game with each other.

Both the Master and the GameHandler are getting every packet pushed into their
VecDeque. -> There could be a structure that looks kind of like this:

```rust
pub struct Distributor<T> {
	receiver: Receiver<T>,
	
}

pub struct ClosedDeque<T> {
	pub (module) inner: VecDeque<T>
}
```
