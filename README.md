# Features
## Move ordering
- MVV-LVA
- SEE
- Transposition table
- Killer moves
- History heuristic

## Pruning
- Transposition tables
- SEE

## Evaluation
- Piece square tables
- Quiescence
    - Delta pruning
    - SEE

# Tests

## Search time increases
### Move ordering
List adds each technique in sequence

#### 7 samples, 6 iterations, depth 6
Alpha-beta + iterative deepen: 1180ms
Transposition table (size 20): 768ms
Simple sorting:                57ms
MVv+LVA:                       33ms
Quiescence:                    70ms
Delta pruning:                 54ms

#### 7 samples, 7 iterations, depth 8
Default:                933ms
SEE quiescence pruning: 993ms (score is higher because SEE is more expensive than current barebones eval) (SEE + 300 < 0)
SEE quiescence pruning: 993ms (score is higher because SEE is more expensive than current barebones eval) (SEE < 0 because stand pat score should be good enough to make sure we don't completely blunder)
SEE alpha-beta pruning: 536ms (SEE < -50)

## Move ordering node count
Better move ordering should result in an increase of prunes from the alpha-beta algorithm, decreasing node count

Tested with tt size of 20, reseting tt each time, depth 9, no pruning (except for tt prune), with quiescence (and delta prune). Each result adds the previous features along with it.
- Default: 2707159010 nodes, 299.81s (9.03 Mnps)
- MVV-LVA: 1934240191 nodes, 151.53s (12.76 Mnps)
- TT:      191103127 nodes, 13s (14.6 Mnps)
- Killer moves: 148118376 nodes, 10.07s (14.72 Mnps)
- History heuristic (with simple gravity): 127350500 nodes, 9.13s (13.94 Mnps)
- SEE: 120827687 nodes, 9.04s (13.36 Mnps)

## STS Suite


# Things to vary
- SEE comparison values (and pruning in general)
- Piece values