
- Theta* will pathfind from the current car's position, to the poisition of a "checkpoint". These will be periodically placed sections along the track
  to ensure the car is moving forward (and not just trying to loop around the finish line). Each checkpoint is a line. Theta* will pathfind to a
  randomized point along that line. This is to ensure some variability in where the AI cars are heading.
  NOTE: These are different than the checkpoints we discussed to ensure you can't skip sections of the track.

- Weights: Each tile type has a weight. For example, the roads have a small weight while grass tiles have a larger weight. Theta* will find the
  lowest cost path from its current point to the next checkpoint.

- Output: Theta* will output a "ThetaCommand." This is an enum that consists of commands such as "TurnLeft", "Forward", etc. These states
  will direct the AI cars to their checkpoints. The states will be checked in a constantly running loop and the commands will be converted into movement
  accordingly.

- Personalities: For our final implementation, we will have different personalities for the AI cars. For example, one personality will shy away
  from other cars (similar to Boids). Another will try to bump into other cars and get in the way. A third will be the normal driver. The shy personality
  will set their checkpoint dynamically away from other players and swerve when needed. Swerving will not be done with theta*. Agressive personality will
  do the exact opposite of the shy one.

Boid examples:
https://en.wikipedia.org/wiki/Boids3,
https://www.youtube.com/watch?v=bqtqltqcQhw