use crate::error_cc::ErrorLevel;
use crate::{ConcentricSquare, encode, Module, Version, ZigzagIter};
use std::collections::HashSet;

#[test]
fn test_concentric_square_iter() {
    let sq = ConcentricSquare {
        center: (3, 3),
        size: 4,
        color_bits: 0b1011,
    };
    let set: HashSet<(u8, u8, bool)> = HashSet::from_iter(sq.iter_squares());
    assert_eq!(set.len(), 49);
    check_contains_module_square(&set, 7u8, (0, 0), true);

    let sq = ConcentricSquare {
        center: (17, 3),
        size: 4,
        color_bits: 0b1011,
    };
    let set: HashSet<(u8, u8, bool)> = HashSet::from_iter(sq.iter_squares());
    assert_eq!(set.len(), 49);

    check_contains_module_square(&set, 7u8, (14, 0), true);
    check_contains_module_square(&set, 5u8, (15, 1), false);
    check_contains_module_square(&set, 3u8, (16, 2), true);
    check_contains_module_square(&set, 1u8, (17, 3), true);

    let sq = ConcentricSquare {
        center: (17, 3),
        size: 3,
        color_bits: 0b1011,
    };
    let set: HashSet<(u8, u8, bool)> = HashSet::from_iter(sq.iter_squares());
    println!("{:?}", &set);
    assert_eq!(set.len(), 25);

    check_contains_module_square(&set, 5u8, (15, 1), false);
}

fn check_contains_module_square(
    set: &HashSet<(u8, u8, bool)>,
    size: u8,
    top_left: (u8, u8),
    is_black: bool,
) {
    let (top_left_x, top_left_y) = top_left;
    for i in 0..size {
        assert_eq!(
            set.contains(&(i + top_left_x, top_left_y, is_black)),
            true,
            "does not contain ({},{}, {})",
            i + top_left_x,
            top_left_y,
            is_black
        );
        assert_eq!(
            set.contains(&(i + top_left_x, top_left_y + size - 1, is_black)),
            true,
            "does not contain ({},{},{})",
            i + top_left_x,
            top_left_y + size - 1,
            is_black
        );
        assert_eq!(
            set.contains(&(top_left_x, i + top_left_y, is_black)),
            true,
            "does not contain ({},{},{})",
            top_left_x,
            i + top_left_y,
            is_black
        );
        assert_eq!(
            set.contains(&(top_left_x + size - 1, i + top_left_y, is_black)),
            true,
            "does not contain ({},{},{})",
            top_left_x + size - 1,
            i + top_left_y,
            is_black
        );
    }
}

#[test]
fn test_alignment_square() {
    assert_eq!(Version(1).alignment_squares_iter().count(), 0);
    let squares: Vec<ConcentricSquare> = Version(2).alignment_squares_iter().collect();
    assert_eq!(squares.len(), 1);
    let square = squares.get(0).unwrap();

    let set: HashSet<(u8, u8, bool)> = HashSet::from_iter(square.iter_squares());
    assert_eq!(true, set.contains(&(17, 17, false)));
    assert_eq!(true, set.contains(&(16, 16, true)));
    assert_eq!(false, set.contains(&(15, 15, true)));
}

#[test]
fn test_version_reserved_area_iter() {
    for v_num in 1..=5 {
        let v = Version(v_num);

        let size = v.square_size();
        let check_reserved_sq = |top_left, bottom_right| {
            let (top_left_x, top_left_y) = top_left;
            let (bottom_right_x, bottom_right_y) = bottom_right;
            for x in top_left_x..=bottom_right_x {
                for y in top_left_y..=bottom_right_y {
                    assert_eq!(
                        false,
                        v.is_data_location((x, y)),
                        "data module location {},{} version={}",
                        x,
                        y,
                        v_num
                    );
                }
            }
        };

        check_reserved_sq((0, 0), (8, 8));
        check_reserved_sq((size - 8, 0), (size - 1, 8));
        check_reserved_sq((0, size - 8), (8, size - 1));
        assert_eq!(true, v.is_data_location((size - 9, 0)));
        assert_eq!(false, v.is_data_location(v.dark_module_location()));
        assert_eq!(false, v.is_data_location((6, 0)));
        assert_eq!(false, v.is_data_location((6, 9)));

        if v_num == 2 {
            //check alignment pattern
            check_reserved_sq((16, 16), (20, 20));
            assert_eq!(
                true,
                v.is_data_location((15, 15)),
                "should be data areas (15,15)"
            );
            assert_eq!(
                true,
                v.is_data_location((21, 21)),
                "should be data areas (15,15)"
            );
        }
        if v_num == 3 {
            //check alignment pattern
            check_reserved_sq((20, 20), (24, 24));
        }
        if v_num == 4 {
            //check alignment pattern
            check_reserved_sq((24, 24), (28, 28));
        }
        if v_num == 5 {
            //check alignment pattern version 5
            check_reserved_sq((28, 28), (32, 32));
        }
    }
}

#[test]
pub fn test_data_region_iter() {
    const VERSION: u8 = 2;
    let mods: Vec<(u8, u8)> = Version(VERSION).data_region_iter().collect();
    let expected_order = &[
        (24u8, 24u8),
        (23, 24),
        (24, 23),
        (23, 23),
        (24, 22),
        (23, 22),
        (24, 21),
        (23, 21),
        (24, 20),
        (23, 20),
        (24, 19),
        (23, 19),
        (24, 18),
        (23, 18),
        (24, 17),
        (23, 17),
        (24, 16),
        (23, 16),
        (24, 15),
        (23, 15),
        (24, 14),
        (23, 14),
        (24, 13),
        (23, 13),
        (24, 12),
        (23, 12),
        (24, 11),
        (23, 11),
        (24, 10),
        (23, 10),
        (24, 9),
        (23, 9),
        (22, 9),
        (21, 9),
        (22, 10),
        (21, 10),
        (22, 11),
        (21, 11),
        (22, 12),
        (21, 12),
        (22, 13),
        (21, 13),
        (22, 14),
        (21, 14),
        (22, 15),
        (21, 15),
        (22, 16),
        (21, 16),
        (22, 17),
        (21, 17),
        (22, 18),
        (21, 18),
        (22, 19),
        (21, 19),
        (22, 20),
        (21, 20),
        (22, 21),
        (21, 21),
        (22, 22),
        (21, 22),
        (22, 23),
        (21, 23),
        (22, 24),
        (21, 24),
        (20, 24),
        (19, 24),
        (20, 23),
        (19, 23),
        (20, 22),
        (19, 22),
        (20, 21),
        (19, 21),
        (20, 15),
        (19, 15),
        (20, 14),
        (19, 14),
        (20, 13),
        (19, 13),
        (20, 12),
        (19, 12),
        (20, 11),
        (19, 11),
        (20, 10),
        (19, 10),
        (20, 9),
        (19, 9),
        (18, 9),
        (17, 9),
        (18, 10),
        (17, 10),
        (18, 11),
        (17, 11),
        (18, 12),
        (17, 12),
        (18, 13),
        (17, 13),
        (18, 14),
        (17, 14),
        (18, 15),
        (17, 15),
        (18, 21),
        (17, 21),
        (18, 22),
        (17, 22),
        (18, 23),
        (17, 23),
        (18, 24),
        (17, 24),
        (16, 24),
        (15, 24),
        (16, 23),
        (15, 23),
        (16, 22),
        (15, 22),
        (16, 21),
        (15, 21),
        (15, 20),
        (15, 19),
        (15, 18),
        (15, 17),
        (15, 16),
        (16, 15),
        (15, 15),
        (16, 14),
        (15, 14),
        (16, 13),
        (15, 13),
        (16, 12),
        (15, 12),
        (16, 11),
        (15, 11),
        (16, 10),
        (15, 10),
        (16, 9),
        (15, 9),
        (16, 8),
        (15, 8),
        (16, 7),
        (15, 7),
        (16, 5),
        (15, 5),
        (16, 4),
        (15, 4),
        (16, 3),
        (15, 3),
        (16, 2),
        (15, 2),
        (16, 1),
        (15, 1),
        (16, 0),
        (15, 0),
        (14, 0),
        (13, 0),
        (14, 1),
        (13, 1),
        (14, 2),
        (13, 2),
        (14, 3),
        (13, 3),
        (14, 4),
        (13, 4),
        (14, 5),
        (13, 5),
        (14, 7),
        (13, 7),
        (14, 8),
        (13, 8),
        (14, 9),
        (13, 9),
        (14, 10),
        (13, 10),
        (14, 11),
        (13, 11),
        (14, 12),
        (13, 12),
        (14, 13),
        (13, 13),
        (14, 14),
        (13, 14),
        (14, 15),
        (13, 15),
        (14, 16),
        (13, 16),
        (14, 17),
        (13, 17),
        (14, 18),
        (13, 18),
        (14, 19),
        (13, 19),
        (14, 20),
        (13, 20),
        (14, 21),
        (13, 21),
        (14, 22),
        (13, 22),
        (14, 23),
        (13, 23),
        (14, 24),
        (13, 24),
        (12, 24),
        (11, 24),
        (12, 23),
        (11, 23),
        (12, 22),
        (11, 22),
        (12, 21),
        (11, 21),
        (12, 20),
        (11, 20),
        (12, 19),
        (11, 19),
        (12, 18),
        (11, 18),
        (12, 17),
        (11, 17),
        (12, 16),
        (11, 16),
        (12, 15),
        (11, 15),
        (12, 14),
        (11, 14),
        (12, 13),
        (11, 13),
        (12, 12),
        (11, 12),
        (12, 11),
        (11, 11),
        (12, 10),
        (11, 10),
        (12, 9),
        (11, 9),
        (12, 8),
        (11, 8),
        (12, 7),
        (11, 7),
        (12, 5),
        (11, 5),
        (12, 4),
        (11, 4),
        (12, 3),
        (11, 3),
        (12, 2),
        (11, 2),
        (12, 1),
        (11, 1),
        (12, 0),
        (11, 0),
        (10, 0),
        (9, 0),
        (10, 1),
        (9, 1),
        (10, 2),
        (9, 2),
        (10, 3),
        (9, 3),
        (10, 4),
        (9, 4),
        (10, 5),
        (9, 5),
        (10, 7),
        (9, 7),
        (10, 8),
        (9, 8),
        (10, 9),
        (9, 9),
        (10, 10),
        (9, 10),
        (10, 11),
        (9, 11),
        (10, 12),
        (9, 12),
        (10, 13),
        (9, 13),
        (10, 14),
        (9, 14),
        (10, 15),
        (9, 15),
        (10, 16),
        (9, 16),
        (10, 17),
        (9, 17),
        (10, 18),
        (9, 18),
        (10, 19),
        (9, 19),
        (10, 20),
        (9, 20),
        (10, 21),
        (9, 21),
        (10, 22),
        (9, 22),
        (10, 23),
        (9, 23),
        (10, 24),
        (9, 24),
        (8, 16),
        (7, 16),
        (8, 15),
        (7, 15),
        (8, 14),
        (7, 14),
        (8, 13),
        (7, 13),
        (8, 12),
        (7, 12),
        (8, 11),
        (7, 11),
        (8, 10),
        (7, 10),
        (8, 9),
        (7, 9),
        (5, 9),
        (4, 9),
        (5, 10),
        (4, 10),
        (5, 11),
        (4, 11),
        (5, 12),
        (4, 12),
        (5, 13),
        (4, 13),
        (5, 14),
        (4, 14),
        (5, 15),
        (4, 15),
        (5, 16),
        (4, 16),
        (3, 16),
        (2, 16),
        (3, 15),
        (2, 15),
        (3, 14),
        (2, 14),
        (3, 13),
        (2, 13),
        (3, 12),
        (2, 12),
        (3, 11),
        (2, 11),
        (3, 10),
        (2, 10),
        (3, 9),
        (2, 9),
        (1, 9),
        (0, 9),
        (1, 10),
        (0, 10),
        (1, 11),
        (0, 11),
        (1, 12),
        (0, 12),
        (1, 13),
        (0, 13),
        (1, 14),
        (0, 14),
        (1, 15),
        (0, 15),
        (1, 16),
        (0, 16),
    ];
    assert_eq!(&mods, &expected_order);
}

#[test]
pub fn test_encode() {
    let code = encode::<128>("isaiah-perumalla").unwrap();

    //ErrorLevel::L
    let expected_words = [
        65, 6, 151, 54, 22, 150, 22, 130, 215, 6, 87, 39, 86, 214, 22, 198, 198, 16, 236, 121,
        93, 100, 23, 7, 230, 143,
    ];
    assert_eq!(&expected_words, code.code_words());
}

#[test]
pub fn test_version_format_modules() {
    let v = Version(1);
    let modules = v.format_modules(ErrorLevel::L, 0);
    assert_eq!(true, modules.iter().all(|m| !m.is_data()));

    println!("{:?}", &modules);
}

