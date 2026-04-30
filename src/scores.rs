pub enum RonOrTsumo {
    Ron(i32),
    Tsumo([i32; 2]),
}

impl From<i32> for RonOrTsumo {
    fn from(value: i32) -> Self {
        RonOrTsumo::Ron(value)
    }
}
impl From<[i32; 2]> for RonOrTsumo {
    fn from(value: [i32; 2]) -> Self {
        RonOrTsumo::Tsumo(value)
    }
}

fn ciel_100(n: i32) -> i32 {
    ((n + 99) / 100) * 100
}

fn calc_basic_points(fu: u32, han: u32, dealer: bool) -> i32 {
    let isdeal = match dealer {
        true => 1,
        false => 0,
    };
    let signed_fu: i32 = fu.try_into().unwrap();
    signed_fu * (2_i32.pow((2 + isdeal) + han))
}

// Multiply number by 1.5 and return as i32 automatically
fn safe_mult(num: i32) -> i32 {
    ((num as f32) * 1.5) as i32
}

pub fn calc_points(
    han: u32,
    fu: u32,
    tsumo: bool,
    dealer: bool,
    kiriage: Option<bool>,
) -> Result<RonOrTsumo, ()> {
    let fu_limit = match (kiriage.is_some_and(|b| b)) {
        true => [30, 60],
        false => [40, 70],
    };
    let resolved_points: RonOrTsumo;

    // Now we reach into limit hands
    if han >= 5 || (han == 4 && fu >= fu_limit[0]) || (han == 3 && fu >= fu_limit[1]) {
        let low_points = if han <= 5 {
            2000
        } else if han <= 7 {
            3000
        } else if han <= 10 {
            4000
        } else if han <= 12 {
            6000
        } else {
            8000
        };

        return if tsumo {
            Ok((match dealer {
                true => [low_points * 2, low_points * 2],
                false => [low_points, low_points*2],
            })
            .into())
        } else {
            Ok((match dealer {
                true => safe_mult(low_points),
                false => low_points,
            } * 4)
                .into())
        };
    }

    // Tsumo
    if tsumo {
        if dealer {
            let tmp = ciel_100(calc_basic_points(fu, han, true));
            resolved_points = [tmp, tmp].into();
        } else {
            let mut out: [i32; 2] = [0, 0];
            out[0] = ciel_100(calc_basic_points(fu, han, false));
            out[1] = ciel_100(calc_basic_points(fu, han, true));
            resolved_points = out.into()
        }
    // Ron
    } else {
        if dealer {
            resolved_points = (ciel_100(calc_basic_points(fu, han, false) * 6)).into()
        } else {
            resolved_points = (ciel_100(calc_basic_points(fu, han, false) * 4)).into()
        }
    }

    Ok(resolved_points)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to unwrap Ron
    fn ron(result: Result<RonOrTsumo, ()>) -> i32 {
        match result.unwrap() {
            RonOrTsumo::Ron(v) => v,
            _ => panic!("Expected Ron"),
        }
    }

    // Helper to unwrap Tsumo as [non_dealer, dealer]
    fn tsumo(result: Result<RonOrTsumo, ()>) -> [i32; 2] {
        match result.unwrap() {
            RonOrTsumo::Tsumo(v) => v,
            _ => panic!("Expected Tsumo"),
        }
    }

    // --- Normal hands ---

    #[test]
    fn test_non_dealer_ron_3han_40fu() {
        // basic = 40 * 2^5 = 1280, * 4 = 5120 -> 5200
        assert_eq!(ron(calc_points(3, 40, false, false, None)), 5200);
    }

    #[test]
    fn test_dealer_ron_3han_40fu() {
        // basic = 40 * 2^5 = 1280, * 6 = 7680 -> 7700
        assert_eq!(ron(calc_points(3, 40, false, true, None)), 7700);
    }

    #[test]
    fn test_non_dealer_tsumo_3han_40fu() {
        // non_dealer = ceil(1280) = 1300, dealer = ceil(2560) = 2600
        assert_eq!(tsumo(calc_points(3, 40, true, false, None)), [1300, 2600]);
    }

    #[test]
    fn test_dealer_tsumo_3han_40fu() {
        // dealer basic = 40 * 2^6 = 2560 -> 2600, both pay same
        assert_eq!(tsumo(calc_points(3, 40, true, true, None)), [2600, 2600]);
    }

    // --- Mangan ---

    #[test]
    fn test_mangan_non_dealer_ron() {
        // 2000 * 4 = 8000
        assert_eq!(ron(calc_points(5, 30, false, false, None)), 8000);
    }

    #[test]
    fn test_mangan_dealer_ron() {
        // 3000 * 4 = 12000
        assert_eq!(ron(calc_points(5, 30, false, true, None)), 12000);
    }

    #[test]
    fn test_mangan_non_dealer_tsumo() {
        assert_eq!(tsumo(calc_points(5, 30, true, false, None)), [2000, 4000]);
    }

    #[test]
    fn test_mangan_dealer_tsumo() {
        assert_eq!(tsumo(calc_points(5, 30, true, true, None)), [4000, 4000]);
    }

    // --- Haneman ---

    #[test]
    fn test_haneman_non_dealer_tsumo() {
        assert_eq!(tsumo(calc_points(6, 30, true, false, None)), [3000, 6000]);
    }

    #[test]
    fn test_haneman_dealer_tsumo() {
        assert_eq!(tsumo(calc_points(6, 30, true, true, None)), [6000, 6000]);
    }

    // --- Kiriage mangan ---

    #[test]
    fn test_kiriage_4han_30fu_is_mangan() {
        // Without kiriage, 4han 30fu is NOT mangan (fu_limit[0] = 40)
        // With kiriage, it IS mangan (fu_limit[0] = 30)
        let without = ron(calc_points(4, 30, false, false, Some(false)));
        let with_kiriage = ron(calc_points(4, 30, false, false, Some(true)));
        assert_ne!(without, 8000);
        assert_eq!(with_kiriage, 8000);
    }

    #[test]
    fn test_kiriage_3han_60fu_is_mangan() {
        let without = ron(calc_points(3, 60, false, false, Some(false)));
        let with_kiriage = ron(calc_points(3, 60, false, false, Some(true)));
        assert_ne!(without, 8000);
        assert_eq!(with_kiriage, 8000);
    }

    // --- Sanbaiman / Yakuman ---

    #[test]
    fn test_baiman_non_dealer_tsumo() {
        assert_eq!(tsumo(calc_points(8, 30, true, false, None)), [4000, 8000]);
    }

    #[test]
    fn test_sanbaiman_non_dealer_tsumo() {
        assert_eq!(tsumo(calc_points(11, 30, true, false, None)), [6000, 12000]);
    }

    #[test]
    fn test_yakuman_non_dealer_tsumo() {
        assert_eq!(tsumo(calc_points(13, 30, true, false, None)), [8000, 16000]);
    }
}
