# Features
## Move ordering
- MVV-LVA
- SEE
- Transposition table
- Killer moves
- History heuristic

## Pruning
- Transposition table
- Null move: Give your opponent a free move, if you're still way stronger than them, prune
- Reverse futility prune: Position is so good that losing a margin still beats beta
- X Futility prune: If the best possible improvement can't raise alpha, position is too bad, don't search

## Reductions
- Late move reduction: Moves later in the movelist are generally worse so search them to a lesser ply
- PVS
- X Internal iterative reduction: If we don't have a TT move, reduce depth by one because without a good first move the search will be inefficient so for that iterative deepening depth

## Etc
- Quiescence
    - Delta pruning
    - SEE pruning
- Iterative deepening
- X Aspiration windows: Narrow alpha-beta serach window around the expected score causing more beta cutoffs, downside is must re-search if node is outside of aspiration window

## Evaluation
- Piece square tables
- Bishop pair
- Open files
- Tapered eval (with ending PST)
- Passed pawns

# Things to vary
- Reverse futility prune, increase RFP_MARGIN and decrease max depth makes more accurate
- Futility prune, increase margin to make more conservative
- LMR, increase moved searched
- SEE comparison values (and pruning in general)
- Piece values
- Positional vs pure piece value