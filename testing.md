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

## Pruning with WAC suite
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

## Evaluation
### Iterative eval
Depth 13, 20 TT
- Pre
    - Median Time: 16.280362353s
    - Median Nodes: 190563485
    - Speed: 11.71 Mnps
- Post
    - Median Time: 13.750168906s
    - Median Nodes: 178678340
    - Speed: 12.99 Mnps

## Etc.
- Full sort
    - Median Time: 13.231288695s
    - Median Nodes: 177297730
    - Speed: 13.40 Mnps
- Incremental selection sort
    - Median Time: 11.989680042s
    - Median Nodes: 175756860
    - Speed: 14.66 Mnps

## Bishop pair
- Pre
    - Median Time: 11.989680042s
    - Median Nodes: 175756860
    - Speed: 14.66 Mnps
- Post
    - Median Time: 15.745234327s
    - Median Nodes: 229843675
    - Speed: 14.60 Mnps
- Post but removing full release speed (from now and on)
    - Median Time: 16.916820527s
    - Median Nodes: 219745785
    - Speed: 12.99 Mnps

Results of bishop_pair vs simple (10+0.1, NULL, NULL, 2moves_v1.epd):
Elo: 45.42 +/- 43.10, nElo: 51.60 +/- 48.15
LOS: 98.22 %, DrawRatio: 57.00 %, PairsRatio: 1.87
Games: 200, Wins: 111, Losses: 85, Draws: 4, Points: 113.0 (56.50 %)
Ptnml(0-2): [13, 2, 57, 2, 26], WL/DD Ratio: inf

## Switch to normal from iterative eval (for simplicity)
- Default (no iterative)
    - Median Time: 16.280362353s
    - Median Nodes: 190563485
    - Speed: 11.71 Mnps
- Bishop pairs
    - Median Time: 22.352756206s
    - Median Nodes: 259449640
    - Speed: 11.61 Mnps

Results of no_iter_bishop_pairs vs no_iter (10+0.1, NULL, NULL, 2moves_v1.epd):
Elo: 55.86 +/- 51.28, nElo: 64.75 +/- 57.97
LOS: 98.57 %, DrawRatio: 56.52 %, PairsRatio: 2.33
Games: 138, Wins: 78, Losses: 56, Draws: 4, Points: 80.0 (57.97 %)
Ptnml(0-2): [8, 1, 39, 3, 18], WL/DD Ratio: inf

Results of no_iter_bishop_pairs vs bishop_pairs (10+0.1, NULL, NULL, 2moves_v1.epd):
Elo: -20.87 +/- 25.49, nElo: -25.05 +/- 30.45
LOS: 5.35 %, DrawRatio: 59.60 %, PairsRatio: 0.71
Games: 500, Wins: 226, Losses: 256, Draws: 18, Points: 235.0 (47.00 %)
Ptnml(0-2): [48, 11, 149, 7, 35], WL/DD Ratio: inf

## Open files
- Pre
    - Median Time: 22.352756206s
    - Median Nodes: 259449640
    - Speed: 11.61 Mnps
- Post
    - Median Time: 17.958562985s
    - Median Nodes: 203897640
    - Speed: 11.35 Mnps

Results of no_iter_open_files vs no_iter_bishop_pairs (10+0.1, NULL, NULL, 2moves_v1.epd):
Elo: 42.60 +/- 28.13, nElo: 46.69 +/- 30.45
LOS: 99.87 %, DrawRatio: 51.60 %, PairsRatio: 1.81
Games: 500, Wins: 271, Losses: 210, Draws: 19, Points: 280.5 (56.10 %)
Ptnml(0-2): [38, 5, 129, 14, 64], WL/DD Ratio: inf

Results of no_iter_open_files vs bishop_pairs (10+0.1, NULL, NULL, 2moves_v1.epd):
Elo: 38.37 +/- 28.42, nElo: 41.56 +/- 30.45
LOS: 99.63 %, DrawRatio: 49.60 %, PairsRatio: 1.62
Games: 500, Wins: 265, Losses: 210, Draws: 25, Points: 277.5 (55.50 %)
Ptnml(0-2): [39, 9, 124, 14, 64], WL/DD Ratio: 123.00

## Unrolled pawns
Pre:
Eval calls: 159716020
Total eval time: 9.22s
Avg eval time: 57.7ns

Post:
Eval calls: 181614775
Total eval time: 6.95s
Avg eval time: 38.3ns

Results of open_files vs no_iter_open_files (10+0.1, NULL, NULL, 2moves_v1.epd):
Elo: 19.48 +/- 19.89, nElo: 21.16 +/- 21.53
LOS: 97.29 %, DrawRatio: 50.80 %, PairsRatio: 1.28
Games: 1000, Wins: 504, Losses: 448, Draws: 48, Points: 528.0 (52.80 %)
Ptnml(0-2): [88, 20, 254, 24, 114], WL/DD Ratio: 126.00

Results of unrolled_pawns_open_files vs open_files (10+0.1, NULL, NULL, 2moves_v1.epd):
Elo: 10.43 +/- 19.73, nElo: 11.40 +/- 21.53
LOS: 85.02 %, DrawRatio: 51.20 %, PairsRatio: 1.18
Games: 1000, Wins: 491, Losses: 461, Draws: 48, Points: 515.0 (51.50 %)
Ptnml(0-2): [94, 18, 256, 28, 104], WL/DD Ratio: 255.00

## Tapered eval
Pre:
Eval calls: 181614775
Total eval time: 6.95s
Avg eval time: 38.3ns

Post:
Eval calls: 210643865
Total eval time: 12.08s
Avg eval time: 57.3ns

Results of tapered_eval vs unrolled_pawns_open_files (10+0.1, NULL, NULL, 2moves_v1.epd):
Elo: 10.77 +/- 20.42, nElo: 11.38 +/- 21.53
LOS: 84.99 %, DrawRatio: 49.00 %, PairsRatio: 1.14
Games: 1000, Wins: 495, Losses: 464, Draws: 41, Points: 515.5 (51.55 %)
Ptnml(0-2): [100, 19, 245, 22, 114], WL/DD Ratio: inf

## Passed pawns
Pre:
Eval calls: 210643865
Total eval time: 12.08s
Avg eval time: 57.3ns

Post:
Eval calls: 235880910
Total eval time: 13.09s
Avg eval time: 55.5ns

Results of passed_pawns vs tapered_eval (10+0.1, NULL, NULL, 2moves_v1.epd):
Elo: 18.88 +/- 20.42, nElo: 21.25 +/- 22.90
LOS: 96.55 %, DrawRatio: 55.20 %, PairsRatio: 1.25
Games: 884, Wins: 450, Losses: 402, Draws: 32, Points: 466.0 (52.71 %)
Ptnml(0-2): [71, 17, 244, 13, 97], WL/DD Ratio: 243.00

Results of passed_pawns vs vault (10+0.1, NULL, NULL, 2moves_v1.epd):
Elo: 33.85 +/- 27.07, nElo: 36.41 +/- 28.88
LOS: 99.33 %, DrawRatio: 37.77 %, PairsRatio: 1.44
Games: 556, Wins: 267, Losses: 213, Draws: 76, Points: 305.0 (54.86 %)
Ptnml(0-2): [41, 30, 105, 38, 64], WL/DD Ratio: 25.25

## Isolated pawns
Pre:
Eval calls: 235880910
Total eval time: 13.09s
Avg eval time: 55.5ns

Post:
Eval calls: 227391790
Total eval time: 13.55s
Avg eval time: 59.6ns

Results of isolated_pawns vs passed_pawns (10+0.1, NULL, NULL, 2moves_v1.epd):
Elo: 9.38 +/- 19.67, nElo: 10.29 +/- 21.53
LOS: 82.54 %, DrawRatio: 54.00 %, PairsRatio: 1.09
Games: 1000, Wins: 498, Losses: 471, Draws: 31, Points: 513.5 (51.35 %)
Ptnml(0-2): [92, 18, 270, 11, 109], WL/DD Ratio: 269.00

## Piece mobility
Pre:
Eval calls: 227391790
Total eval time: 13.55s
Avg eval time: 59.6ns

Post:
Eval calls: 297412200
Total eval time: 30.53s
Avg eval time: 102.6ns

Results of mobility vs isolated_pawns (10+0.1, NULL, NULL, 2moves_v1.epd):
Elo: 18.87 +/- 20.95, nElo: 20.92 +/- 23.14
LOS: 96.18 %, DrawRatio: 56.12 %, PairsRatio: 1.26
Games: 866, Wins: 448, Losses: 401, Draws: 17, Points: 456.5 (52.71 %)
Ptnml(0-2): [74, 10, 243, 7, 99], WL/DD Ratio: inf

Post aggression:
Eval calls: 240334185
Total eval time: 25.29s
Avg eval time: 105.2ns

Results of mobility_aggression vs mobility (10+0.1, NULL, NULL, 2moves_v1.epd):
Elo: -3.82 +/- 22.51, nElo: -4.05 +/- 23.81
LOS: 36.94 %, DrawRatio: 50.12 %, PairsRatio: 0.94
Games: 818, Wins: 390, Losses: 399, Draws: 29, Points: 404.5 (49.45 %)
Ptnml(0-2): [89, 16, 205, 13, 86], WL/DD Ratio: inf
