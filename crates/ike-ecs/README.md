# Ike ECS
Ike at it's core uses an entity component system for managing the moving parts of the game engine.
The reasons for which are plentiful and are explained in [motivations](#motivations).

## Concepts
A more detailed explanation of entity component systems, written by Ian Kettlewell, 
can be found [here](https://ianjk.com/ecs-in-rust/).

Unlike most ECS' out there we have locks on each component.
While they introduce a small memory overhead they allow us to do things like [nodes](../ike-node/),
which allows for a code structure more familiar to most game developers.

## Motivations

## Inspirations
Everyone build on the shoulders of others. Here is a list of inspirations for our ECS implementation.
 * [bevy](https://bevyengine.org/)
 * [hecs](https://docs.rs/hecs/latest/hecs/)
 * [legion](https://docs.rs/legion/0.4.0/legion/)