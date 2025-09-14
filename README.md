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

Rust Racers is a top down, 2D racing game. Players can queue up in a lobby to start the race. During the race, players can control their cars with WASD and will drive around pre-created tracks. Rust Racers will support up to 8 players and will fill in non-player slots with AI racers.

## Advanced Topic Description

### Multiplayer Networking

Rust Racers will include a custom networking system that allows players to compete together in real time. The game will feature a lobby where players can create or join rooms before starting a race, supporting up to eight participants in a single session. Once the race begins, the networking logic will synchronize game state across all connected players to ensure a consistent experience. The system will handle the transmission of player inputs and game events, maintaining smooth gameplay while minimizing any desynchronization.

### AI Racers

Users will be able to fill in empty player slots with AI computer racers. These racers will have customizable difficulty in order to accommodate for different player skill levels. The CPU players will be able to navigate around the maps, avoid obstacles and drift around corners. Additionally, there will be multiple types of CPU behavior. Some will focus on racing while others will gang up and try to attack players/other CPUs. As a part of the powerup stretch goal, the CPUs will be able to use the various powerups and integrate them into their attack style. 

## Midterm Goals

* Basic Movement
* Game physics
  * Acceleration
  * Collisions
  * Slow down and speed up on different terrain
* Camera tracking
* One track and one car model

## Final Goals

* 50%: Networked Multiplayer and AI CPUs
  * A homescreen with create and join room options
  * Players can choose from multiple cars
  * Players can race alongside a max of 7 players via a networked connection
  * Empty spots within a lobby are filled by AI CPUs
  * Players can also choose to race alone alongside AI CPUs (singleplayer)
  * AI CPUs have dynamic racing behavior that reacts to their surroundings
    * Aggressive Driving Lines
    * Reversing
    * Attacking Players / Other CPUs

* 15%: Two Maps
  * Players can choose between two maps to race on
* 15%: Drifting AND Simple and Complex control styles
  * Players can drift their cars around turns for a small boost
  * Certain driving features are disabled when the player selects simple controls

## Stretch Goals

* Powerups
  * Players and CPUs can collect items (e.g. speed boosts, projectiles) to gain advantages or disrupt others
  * Powerups integrate into both racing-focused and aggressive AI behavior
* Drifting
  * Add drifting mechanics that reward precise timing and skillful cornering
