# Rust Racers

by Six Seven Studios

## Team Members

* Advanced Topic Subteam 1: Multiplayer Networking
  * kaj143: Kameren Jouhal
  * jac608: Jonathan Coulter
  * dzs19: David Shi
  * jdl137: Jeremy Luu

* Advanced Topic Subteam 2: AI Racer
  * clg129: Carson Gollinger
  * gjb46: Greyson Barsotti
  * dsc60: Daniel Cheng
  * ecd57: Ethan Defilippi

## Game Description

Rust Racers is a top down, 2D racing game. Players can queue up in a lobby to start the race. During the race, players can control their cars with WASD and will drive around pre-created tracks. Rust Racers will support up to 4 players and will fill in non-player slots with AI racers.

## Advanced Topic Description

### Multiplayer Networking

Rust Racers will include a custom networking system that allows players to compete together in real time. The game will feature a lobby where players can create or join rooms before starting a race, supporting up to four participants in a single session. Once the race begins, the networking logic will synchronize game state across all connected players to ensure a consistent experience.

**Network Architecture:**
The system will use a client-server model. Player inputs and car physics updates will be transmitted between clients to maintain synchronized gameplay.

**Lag Compensation Strategy:**
To address network latency issues common in real-time multiplayer games, we will implement client-side prediction as our lag compensation technique:

- **Client-side prediction**: Players' own cars will respond immediately to input without waiting for server confirmation, so the game feels responsive. Additionally, we will handle server corrections smoothly to avoid snappy adjustements in car positions.

### AI Racers

Users will be able to fill in empty player slots with AI computer racers. These racers will have customizable difficulty in order to accommodate for different player skill levels. The CPU players will be able to navigate around the maps, avoid obstacles and drift around corners. Additionally, there will be multiple types of CPU behavior. Some will focus on racing while others will gang up and try to disrupt players/other CPUs. As a part of the powerup stretch goal, the CPUs will be able to use the powerups and integrate them into their play style.

## Midterm Goals

* Basic Movement
  * Using WASD to be able to make the car accelerate, slow down, reverse, and turn
* Game physics
  * Acceleration
  * Collisions
    * When cars collide, they will bounce apart, gaining a small instant change in velocity away from the other car
  * Different types of terrain
    * This will include the track and grass which will slow down the cars
* Camera tracking
* One track and one car model
  * The track will be sized such that it takes about 30 seconds to do one lap


## Final Goals

* 35% - Networked Multiplayer
  * 5% - A homescreen with create and join room options
  * 20% - Players can race alongside a max of 3 players via a networked connection
  * 10% - Lag compensation (client-side prediction)

* 35% - AI CPUs
   * Empty spots within a lobby are filled by AI CPUs
   * Players can also choose to race alone alongside AI CPUs (singleplayer)
   * AI CPUs have dynamic racing behavior that reacts to their surroundings implemented with the Theta* algorithm
   * Aggressive Driving Lines
   * Reversing
   * Attacking Players / Other CPUs

* 10% - Two Maps
  * Players can choose between two maps to race on


## Stretch Goals

* Powerups
  * Add one item you can collect as you race (likely a speed boost)
  * Racers may only have 1 power up at a time and can activate it with a key press
* Drifting + Simple driving
  * Players can drift their cars around turns for a small boost
  * Simple setting makes drifting easier when the player enables it
