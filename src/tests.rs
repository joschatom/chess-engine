/// PERFT(depth) tests


#[cfg(test)]
mod perft{
    use crate::{board::Board, utils};


    #[test]
    pub fn startpos() {
        const RESULTS: [u64; 5] = [
            1,
            20,
            400,
            8902,
            197281,
        ];

        let mut board = Board::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_owned())
            .expect("failed to construct board with starting position");


        for depth in 1..=4 {
            eprintln!("Running Perft({})...", depth);

            let res = utils::perft(&mut board, depth, depth, &mut None);

            assert_eq!(res, RESULTS[depth as usize], "Perft({}) returned an incorrect value.", depth);
        }
    }
}

