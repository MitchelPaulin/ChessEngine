pub use crate::board::*;
pub use crate::board::{PieceColor::*, PieceKind::*};
pub use crate::move_generation::*;
use std::cmp;

/*
    Evaluation function based on https://www.chessprogramming.org/Simplified_Evaluation_Function
*/

static PAWN_WEIGHTS: [[i32; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [50, 50, 50, 50, 50, 50, 50, 50],
    [10, 10, 20, 30, 30, 20, 10, 10],
    [5, 5, 10, 25, 25, 10, 5, 5],
    [0, 0, 0, 20, 20, 0, 0, 0],
    [5, -5, -10, 0, 0, -10, -5, 5],
    [5, 10, 10, -20, -20, 10, 10, 5],
    [0, 0, 0, 0, 0, 0, 0, 0],
];

static KNIGHT_WEIGHTS: [[i32; 8]; 8] = [
    [-50, -40, -30, -30, -30, -30, -40, -50],
    [-40, -20, 0, 0, 0, 0, -20, -40],
    [-30, 0, 10, 15, 15, 10, 0, -30],
    [-30, 5, 15, 20, 20, 15, 5, -30],
    [-30, 0, 15, 20, 20, 15, 0, -30],
    [-30, 5, 10, 15, 15, 10, 5, -30],
    [-40, -20, 0, 5, 5, 0, -20, -40],
    [-50, -40, -30, -30, -30, -30, -40, -50],
];

static BISHOP_WEIGHTS: [[i32; 8]; 8] = [
    [-20, -10, -10, -10, -10, -10, -10, -20],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 10, 10, 5, 0, -10],
    [-10, 5, 5, 10, 10, 5, 5, -10],
    [-10, 0, 10, 10, 10, 10, 0, -10],
    [-10, 10, 10, 10, 10, 10, 10, -10],
    [-10, 5, 0, 0, 0, 0, 5, -10],
    [-20, -10, -10, -10, -10, -10, -10, -20],
];

static ROOK_WEIGHTS: [[i32; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [5, 10, 10, 10, 10, 10, 10, 5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [0, 0, 0, 5, 5, 0, 0, 0],
];

static QUEEN_WEIGHTS: [[i32; 8]; 8] = [
    [-20, -10, -10, -5, -5, -10, -10, -20],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 5, 5, 5, 0, -10],
    [-5, 0, 5, 5, 5, 5, 0, -5],
    [0, 0, 5, 5, 5, 5, 0, -5],
    [-10, 5, 5, 5, 5, 5, 0, -10],
    [-10, 0, 5, 0, 0, 0, 0, -10],
    [-20, -10, -10, -5, -5, -10, -10, -20],
];

static KING_WEIGHTS: [[i32; 8]; 8] = [
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-20, -30, -30, -40, -40, -30, -30, -20],
    [-10, -20, -20, -20, -20, -20, -20, -10],
    [20, 20, 0, 0, 0, 0, 20, 20],
    [20, 30, 10, 0, 0, 10, 30, 20],
];

static KING_LATE_GAME: [[i32; 8]; 8] = [
    [-50, -40, -30, -20, -20, -30, -40, -50],
    [-30, -20, -10, 0, 0, -10, -20, -30],
    [-30, -10, 20, 30, 30, 20, -10, -30],
    [-30, -10, 30, 40, 40, 30, -10, -30],
    [-30, -10, 30, 40, 40, 30, -10, -30],
    [-30, -10, 20, 30, 30, 20, -10, -30],
    [-30, -30, 0, 0, 0, 0, -30, -30],
    [-50, -30, -30, -30, -30, -30, -30, -50],
];

fn get_pos_evaluation(row: usize, col: usize, board: &BoardState, color: PieceColor) -> i32 {
    if let Square::Full(piece) = board.board[row][col] {
        let col = col - BOARD_START;
        let row = match color {
            PieceColor::White => row - BOARD_START,
            _ => 9 - row,
        };

        match piece.kind {
            Pawn => PAWN_WEIGHTS[row][col],
            Rook => ROOK_WEIGHTS[row][col],
            Bishop => BISHOP_WEIGHTS[row][col],
            Knight => KNIGHT_WEIGHTS[row][col],
            King => {
                if board.full_move_clock > 30 {
                    KING_LATE_GAME[row][col]
                } else {
                    KING_WEIGHTS[row][col]
                }
            }
            Queen => QUEEN_WEIGHTS[row][col],
        }
    } else {
        panic!("Could not recognize piece")
    }
}

/*
    Return a number to represent how good a certain position is

    White will attempt to "maximize" this score while black will attempt to "minimize" it
*/
pub fn get_evaluation(board: &BoardState) -> i32 {
    let mut evaluation = board.white_total_piece_value;
    evaluation -= board.black_total_piece_value;
    for row in BOARD_START..BOARD_END {
        for col in BOARD_START..BOARD_END {
            let square = board.board[row][col];
            if let Square::Full(Piece { color, .. }) = square {
                let square_eval = get_pos_evaluation(row, col, board, color);
                if color == White {
                    evaluation += square_eval;
                } else {
                    evaluation -= square_eval;
                }
            }
        }
    }
    evaluation
}

/*
    Run a standard alpha beta search to try and find the best move searching up to 'depth'
    Orders moves by piece value to attempt to improve search efficiency
*/
pub fn alpha_beta_search(
    board: &BoardState,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: PieceColor,
) -> (Option<BoardState>, i32) {
    if depth == 0 {
        return (None, get_evaluation(board));
    }

    let mut moves = generate_moves(board);

    if moves.is_empty() {
        // here we add the depths to encourage faster checkmates
        if maximizing_player == PieceColor::White {
            if is_check(board, PieceColor::White) {
                return (None, -99999999 - depth as i32); // checkmate
            }
        } else if is_check(board, PieceColor::Black) {
            return (None, 99999999 + depth as i32); // checkmate
        }
        return (None, 0); // stalemate
    }

    let mut best_move = None;
    if maximizing_player == PieceColor::White {
        moves.sort_by(|a, b| piece_value_differential(b).cmp(&piece_value_differential(a)));
        let mut best_val = i32::MIN;
        for board in moves {
            let evaluation = alpha_beta_search(&board, depth - 1, alpha, beta, PieceColor::Black);
            if evaluation.1 > best_val {
                best_val = evaluation.1;
                best_move = Some(board);
            }
            alpha = cmp::max(alpha, evaluation.1);
            if beta <= alpha {
                break;
            }
        }
        (best_move, best_val)
    } else {
        moves.sort_by(|a, b| piece_value_differential(a).cmp(&piece_value_differential(b)));
        let mut best_val = i32::MAX;
        for board in moves {
            let evaluation = alpha_beta_search(&board, depth - 1, alpha, beta, PieceColor::White);
            if evaluation.1 < best_val {
                best_val = evaluation.1;
                best_move = Some(board);
            }
            beta = cmp::min(beta, evaluation.1);
            if beta <= alpha {
                break;
            }
        }
        (best_move, best_val)
    }
}

fn piece_value_differential(board: &BoardState) -> i32 {
    board.white_total_piece_value - board.black_total_piece_value
}

/*
    Play a game in the terminal where the engine plays against itself
*/
pub fn play_game_against_self(b: &BoardState, depth: u8, max_moves: u8, simple_print: bool) {
    let mut board = b.clone();

    let show_board = |simple_print: bool, b: &BoardState| {
        if simple_print {
            b.simple_print_board()
        } else {
            b.pretty_print_board()
        }
    };

    show_board(simple_print, &board);
    while board.full_move_clock < max_moves {
        let res = alpha_beta_search(&board, depth, i32::MIN, i32::MAX, board.to_move);
        if res.0.is_some() {
            board = res.0.unwrap().clone();
        } else {
            break;
        }
        show_board(simple_print, &board);
    }
}
