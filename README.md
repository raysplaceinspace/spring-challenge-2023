# CodinGame Spring Challenge 2023

**raysplaceinspace**'s submission to the CodinGame Spring Challenge 2023. This entry made it into the Legend League (top 100) out of 5000 players.

The general approach is an optimization algorithm generating plans which are evaluated by running a simulator of the game.
The optimization algorithm is both an ant colony optimization algorithm and a local search algorithm. This was done because the local search algorithm by itself gets stuck in local minima - generally it only can make minor variations on the first plan. The ant colony algorithm can search more novel solutions and find bigger improvements. Generally the first few turns will benefit a lot from the more global search of the ant colony algorithm, then the remaining turns will benefit more from the more detailed improvements of the local search algorithm. The optimization algorithm learns which subalgorithm is producing better results and gives it a higher proportion of its time so this is tuned automatically.

The plans consist of a sequence of cells to visit. It does not contain every single action that needs to be taken - it is a much higher level representation of the plan. This was done so the optimization algorithm could focus on the bigger picture as this has more impact on improving the result. Otherwise there would be too many combinations to explore and the algorithm would get lost in the details. Pathfinding is done using Djikstra's algorithm.

The simulator spends 10 milliseconds each tick optimizing a plan for the enemy, and then 80 milliseconds optimizing our plan. Since they are quite high-level, the plans can be reused between turns, which means we benefit from the growing number of past iterations each turn. The ant colony's pheromone matrix also is preserved between turns and so that also benefits in the same way. This approach is useful because of its simplicity - the same code gets reused for both the opponent and ourselves. The trouble with this approach is we are assuming the opponent will only make one particular move and we overfit for that enemy. For example, sometimes we generate a plan of our opponent changing focus, leaving a particular cell behind, which means our bot decides it doesn't need to fight as hard for that cell anymore. When in reality, the opponent continues to fight. It would have been prudent for us to go all in, whether or not the opponent continues to fight, but our bot is only assessing a single future at a time and so cannot make a decision integrating over the probabilistic spectrum of potential countermoves. Perhaps a more advanced algorithm like Monte Carlo Tree Search might be able to propagate multiple futures back against our actions and help us make smarter moves.

This solution was written in Rust, which as always, continues to be a delightful language for its combination of performance and expressiveness.


# How to run

## Prerequisites

1. Install cargo watch: `cargo install cargo-watch`
2. Install cargo merge: `cargo install cargo-merge`

## Development

1. Run `./start.sh` to watch the source code and merge it each time
2. Use the CodinGame sync Chrome extension to sync `/target/merge/merged.rs` into CodinGame