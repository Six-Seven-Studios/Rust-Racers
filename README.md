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

**Client-side prediction**: Players' own cars will respond immediately to input without waiting for server confirmation, so the game feels responsive. Additionally, we will handle server corrections smoothly to avoid snappy adjustments in car positions.

### AI Racers

Users will be able to fill in empty player slots with AI computer racers. These racers will have customizable difficulty in order to accommodate for different player skill levels. The CPU players will be able to navigate around the maps, avoid obstacles and drift around corners. Additionally, there will be multiple types of CPU behavior. Some will focus on racing while others will gang up and try to disrupt players/other CPUs. As a part of the power-up stretch goal, the CPUs will be able to use the power-ups and integrate them into their play style.

**Advanced AI Algorithm:**
Using the Theta* algorithm, our CPU racers will be able to provide a real challenge to our human players. We chose Theta* because it has a good balance of performance, and adaptability to the track conditions. This algorithm will be able to adjust driving angles on the fly in order to give the CPUs extremely competitive driving lines. 

**Power-Up Usage:**
Using a decision tree, our CPUs will be able to quickly adjust between driving normally  and using power-ups for an advantage.  


## Midterm Goals

* Basic Movement
  * Using WASD to be able to make the car accelerate, slow down, reverse, and turn
* Game physics
  * Acceleration
  * Collisions
    * When cars collide, they will bounce apart, gaining a small instant change in velocity away from the other car
  * 2 Different types of terrain: track and grass
    * Grass which will slow down the cars
    * Driving on track will be faster than driving on grass
  * One map of size 5000px x 5000px
  * We will have at least one car model
    * By car model we mean a car skin

## Final Goals

* 35% - Networked Multiplayer
  * 5% - A home-screen with create and join room options
  * 20% - Players can race alongside a max of 3 players via a networked connection
  * 10% - Lag compensation (client-side prediction)

* 35% - AI CPUs
  * 5% - Empty spots within a lobby are filled by AI CPUs
  * 5% - Players can also choose to race alone alongside AI CPUs (single-player)
  * 15% - AI CPUs have dynamic racing behavior that reacts to their surroundings implemented with the Theta* algorithm, including reversing when needed and aggressive driving lines.
  * 10% - Attacking players / other CPUs decided by a decision tree

* 10% - Secondary Map and Skins
  * 5% - Players can choose between two maps to race on: The original 5000px x 5000px map, and a larger, more difficult 8000px x 8000px map. The secondary map will test player skill by adding a higher density of obstacles and incorporating all of our terrain types.
  * 5% - We will also have one car model/skin for each member of our team

## Stretch Goals

* Power-ups
  * Add one item you can collect as you race (likely a speed boost)
  * Racers may only have 1 power up at a time and can activate it with a key press
* Drifting + Simple driving
  * Players can drift their cars around turns for a small boost
  * Simple setting makes drifting easier when the player enables it
