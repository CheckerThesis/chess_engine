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
- Futility prune: If the best possible improvement can't raise alpha, position is too bad, don't search

## Reductions
- Late move reduction: Moves later in the movelist are generally worse so search them to a lesser ply
- PVS

## Etc
- Quiescence
    - Delta pruning
    - SEE pruning
- Iterative deepening

## Evaluation
- Piece square tables

# Tests
## Move ordering node count
Better move ordering should result in an increase of prunes from the alpha-beta algorithm, decreasing node count

Tested with tt size of 20, reseting tt each time, depth 9, no pruning (except for tt prune), with quiescence (and delta prune). Each result adds the previous features along with it.
- Default: 2707159010 nodes, 299.81s (9.03 Mnps)
- MVV-LVA: 1934240191 nodes, 151.53s (12.76 Mnps)
- TT:      191103127 nodes, 13s (14.6 Mnps)
- Killer moves: 148118376 nodes, 10.07s (14.72 Mnps)
- History heuristic (with simple gravity): 127350500 nodes, 9.13s (13.94 Mnps)
- SEE:         120827687 nodes, 9.04s (13.36 Mnps)
- Countermove: 123477893 nodes, 9.30s (12.98 Mnps) 

## Evaluation STS suite
With quiescence (and delta prune)
- TT 
    - Final Score: 245/300 (81.7%)
    - Median Time: 132.408111985s
    - Median Nodes: 835608048
    - Speed: 6.31 Mnps
- Null move prune (R = 3) (will not continue with)
    - Final Score: 245/300 (81.7%)
    - Median Time: 10.597763619s
    - Median Nodes: 68084214
    - Speed: 6.42 Mnps
- Null move prune (R = 4)
    - Final Score: 238/300 (79.3%)
    - Median Time: 7.374740213s
    - Median Nodes: 49764980
    - Speed: 6.75 Mnps
- Reverse futility prune (margin 120, depth 6) (will not continue with)
    - Final Score: 220/300 (73.3%)
    - Median Time: 1.27778892s
    - Median Nodes: 13357720
    - Speed: 10.45 Mnps
- Reverse futility prune (margin 120, depth 4) (will not continue with)
    - Final Score: 225/300 (75.0%)
    - Median Time: 1.377877393s
    - Median Nodes: 13809616
    - Speed: 10.02 Mnps
- Reverse futility prune (margin [0, 100, 180, 260, 340, 420, 500], depth 4)
    - Final Score: 230/300 (76.7%)
    - Median Time: 1.52705463s
    - Median Nodes: 16096088
    - Speed: 10.54 Mnps
- Late move reduction (moves searched 6, depth 2, reduction, >= 18 ? 2 : 1)
    - Final Score: 223/300 (74.3%)
    - Median Time: 728.625984ms
    - Median Nodes: 7992482
    - Speed: 10.54 Mnps
- PVS
    - Final Score: 215/300 (71.7%)
    - Median Time: 606.397042ms
    - Median Nodes: 6410716
    - Speed: 10.57 Mnps

# Things to vary
- Reverse futility prune, increase RFP_MARGIN and decrease max depth makes more accurate
- Futility prune, increase margin to make more conservative
- LMR, increase moved searched
- SEE comparison values (and pruning in general)
- Piece values