use std::sync::LazyLock;

use rand::prelude::*;
use serde::{Deserialize, Serialize};

static RAW_SCORES_STRING: &str = include_str!("../data/score_probabilities.json");
static RAW_PROBABILITIES: LazyLock<Vec<RawProbability>> =
    LazyLock::new(|| serde_json::from_str(RAW_SCORES_STRING).unwrap());

// Returns a weighted list of scores and their probabilities.
//
// `param` will let you control how common or uncommon unlikely hands are.
//  - `0.0` makes all hands equally likely
//  - `1.0` uses raw frequencies from tenhou data.
//
// let probs = get_probabilities(0.5);
// assert!(!probs.is_empty());
pub fn get_probabilities(param: f32) -> Vec<Probability> {
    let mut out: Vec<Probability> = Vec::new();

    for raw in (*RAW_PROBABILITIES).iter() {
        out.push(Probability {
            score: raw.score,
            probability: (raw.count as f32).powf(param),
        });
    }

    out
}

pub fn random_score(rng: &mut impl Rng, param: f32) -> Score {
    let probs = get_probabilities(param);
    let mut total: f32 = 0.0;
    for p in &probs {
        total += p.probability;
    }

    let mut roll = rng.random_range(0.0..total);

    for s in &probs {
        let weight = s.probability;
        if roll < weight {
            return s.score;
        }
        roll -= weight;
    }

    probs.last().unwrap().score

}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Probability {
    pub score: Score,
    pub probability: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RawProbability {
    pub score: Score,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Score {
    pub han: u32,
    pub fu: u32,
}

#[derive(Debug)]
pub enum RonOrTsumo {
    Ron(i32),
    Tsumo([i32; 2]),
}

impl ToString for RonOrTsumo {
    fn to_string(&self) -> String {
        match self {
            RonOrTsumo::Ron(r) => r.to_string(),
            RonOrTsumo::Tsumo(t) => match t[0] == t[1] {
                true => format!("{} all", t[0]),
                false => format!("{}/{}", t[0], t[1]),
            },
        }
    }
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

impl Score {
    pub fn points(&self, tsumo: bool, dealer: bool, kiriage: bool) -> Result<RonOrTsumo, ()> {
        {
            let fu_limit = match (kiriage) {
                true => [30, 60],
                false => [40, 70],
            };
            let resolved_points: RonOrTsumo;

            // Now we reach into limit hands
            if self.han >= 5
                || (self.han == 4 && self.fu >= fu_limit[0])
                || (self.han == 3 && self.fu >= fu_limit[1])
            {
                let low_points = if self.han <= 5 {
                    2000
                } else if self.han <= 7 {
                    3000
                } else if self.han <= 10 {
                    4000
                } else if self.han <= 12 {
                    6000
                } else {
                    8000
                };

                return if tsumo {
                    Ok((match dealer {
                        true => [low_points * 2, low_points * 2],
                        false => [low_points, low_points * 2],
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
                    let tmp = ciel_100(calc_basic_points(self.fu, self.han, true));
                    resolved_points = [tmp, tmp].into();
                } else {
                    let mut out: [i32; 2] = [0, 0];
                    out[0] = ciel_100(calc_basic_points(self.fu, self.han, false));
                    out[1] = ciel_100(calc_basic_points(self.fu, self.han, true));
                    resolved_points = out.into()
                }
            // Ron
            } else {
                if dealer {
                    resolved_points =
                        (ciel_100(calc_basic_points(self.fu, self.han, false) * 6)).into()
                } else {
                    resolved_points =
                        (ciel_100(calc_basic_points(self.fu, self.han, false) * 4)).into()
                }
            }

            Ok(resolved_points)
        }
    }
}

pub fn score(han: u32, fu: u32) -> Score {
    Score { han, fu }
}
pub fn get_ron(result: Result<RonOrTsumo, ()>) -> i32 {
    match result.unwrap() {
        RonOrTsumo::Ron(v) => v,
        _ => panic!("Expected Ron"),
    }
}

pub fn get_tsumo(result: Result<RonOrTsumo, ()>) -> [i32; 2] {
    match result.unwrap() {
        RonOrTsumo::Tsumo(v) => v,
        _ => panic!("Expected Tsumo"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ron(result: Result<RonOrTsumo, ()>) -> i32 {
        match result.unwrap() {
            RonOrTsumo::Ron(v) => v,
            _ => panic!("Expected Ron"),
        }
    }

    fn tsumo(result: Result<RonOrTsumo, ()>) -> [i32; 2] {
        match result.unwrap() {
            RonOrTsumo::Tsumo(v) => v,
            _ => panic!("Expected Tsumo"),
        }
    }

    // --- Normal hands ---

    #[test]
    fn test_non_dealer_ron_3han_40fu() {
        assert_eq!(
            ron(Score { han: 3, fu: 40 }.points(false, false, false)),
            5200
        );
    }

    #[test]
    fn test_dealer_ron_3han_40fu() {
        assert_eq!(
            ron(Score { han: 3, fu: 40 }.points(false, true, false)),
            7700
        );
    }

    #[test]
    fn test_non_dealer_tsumo_3han_40fu() {
        assert_eq!(
            tsumo(Score { han: 3, fu: 40 }.points(true, false, false)),
            [1300, 2600]
        );
    }

    #[test]
    fn test_dealer_tsumo_3han_40fu() {
        assert_eq!(
            tsumo(Score { han: 3, fu: 40 }.points(true, true, false)),
            [2600, 2600]
        );
    }

    // --- Mangan ---

    #[test]
    fn test_mangan_non_dealer_ron() {
        assert_eq!(
            ron(Score { han: 5, fu: 30 }.points(false, false, false)),
            8000
        );
    }

    #[test]
    fn test_mangan_dealer_ron() {
        assert_eq!(
            ron(Score { han: 5, fu: 30 }.points(false, true, false)),
            12000
        );
    }

    #[test]
    fn test_mangan_non_dealer_tsumo() {
        assert_eq!(
            tsumo(Score { han: 5, fu: 30 }.points(true, false, false)),
            [2000, 4000]
        );
    }

    #[test]
    fn test_mangan_dealer_tsumo() {
        assert_eq!(
            tsumo(Score { han: 5, fu: 30 }.points(true, true, false)),
            [4000, 4000]
        );
    }

    // --- Haneman ---

    #[test]
    fn test_haneman_non_dealer_tsumo() {
        assert_eq!(
            tsumo(Score { han: 6, fu: 30 }.points(true, false, false)),
            [3000, 6000]
        );
    }

    #[test]
    fn test_haneman_dealer_tsumo() {
        assert_eq!(
            tsumo(Score { han: 6, fu: 30 }.points(true, true, false)),
            [6000, 6000]
        );
    }

    // --- Kiriage mangan ---

    #[test]
    fn test_kiriage_4han_30fu_is_mangan() {
        assert_ne!(
            ron(Score { han: 4, fu: 30 }.points(false, false, false)),
            8000
        );
        assert_eq!(
            ron(Score { han: 4, fu: 30 }.points(false, false, true)),
            8000
        );
    }

    #[test]
    fn test_kiriage_3han_60fu_is_mangan() {
        assert_ne!(
            ron(Score { han: 3, fu: 60 }.points(false, false, false)),
            8000
        );
        assert_eq!(
            ron(Score { han: 3, fu: 60 }.points(false, false, true)),
            8000
        );
    }

    // --- Baiman / Sanbaiman / Yakuman ---

    #[test]
    fn test_baiman_non_dealer_tsumo() {
        assert_eq!(
            tsumo(Score { han: 8, fu: 30 }.points(true, false, false)),
            [4000, 8000]
        );
    }

    #[test]
    fn test_sanbaiman_non_dealer_tsumo() {
        assert_eq!(
            tsumo(Score { han: 11, fu: 30 }.points(true, false, false)),
            [6000, 12000]
        );
    }

    #[test]
    fn test_yakuman_non_dealer_tsumo() {
        assert_eq!(
            tsumo(Score { han: 13, fu: 30 }.points(true, false, false)),
            [8000, 16000]
        );
    }
    #[test]
    fn test_ron_string() {
        assert_eq!(RonOrTsumo::Ron(5200).to_string(), "5200")
    }
    #[test]
    fn test_dealer_tsumo_3han_40fu_string() {
        assert_eq!(RonOrTsumo::Tsumo([2600, 2600]).to_string(), "2600 all")
    }
    #[test]
    fn test_non_dealer_tsumo_3han_40fu_string() {
        assert_eq!(RonOrTsumo::Tsumo([1300, 2600]).to_string(), "1300/2600")
    }
}
