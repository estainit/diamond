#[cfg(test)]
pub mod tests_logarithm {
    use crate::{cutils};

    #[test]
    pub fn test_logarithm_1() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(0, 100, 1);
        assert_eq!(gain, 100.0);
    }

    #[test]
    pub fn test_logarithm_2() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(50, 100, 1);
        assert_eq!(gain, 84.94850021680094);
    }

    #[test]
    pub fn test_logarithm_3() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(95, 100, 1);
        assert_eq!(gain, 34.94850021680094);
    }

    #[test]
    pub fn test_logarithm_4() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(96, 100, 1);
        assert_eq!(gain, 30.102999566398122);
    }

    #[test]
    pub fn test_logarithm_5() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(97, 100, 1);
        assert_eq!(gain, 23.85606273598312);
    }

    #[test]
    pub fn test_logarithm_6() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(98, 100, 1);
        assert_eq!(gain, 15.051499783199061);
    }

    #[test]
    pub fn test_logarithm_7() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(99, 100, 1);
        assert_eq!(gain, 0.0);
    }

    #[test]
    pub fn test_logarithm_8() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(100, 100, 1);
        assert_eq!(gain, 0.0);
    }

    // for example for election in 2 day
    pub const TIME_FRAME: i64 = 2 * 24 * 60;
    pub const NEGATIVE_TIME_FRAME: i64 = TIME_FRAME + (TIME_FRAME / 2);

    #[test]
    pub fn test_logarithm_10() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(0, TIME_FRAME, 1);
        assert_eq!(gain, 100.0);
    }

    #[test]
    pub fn test_logarithm_11() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(TIME_FRAME / 4, TIME_FRAME, 1);
        assert_eq!(gain, 96.38841972830821);
    }

    #[test]
    pub fn test_logarithm_12() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(TIME_FRAME / 2, TIME_FRAME, 1);
        assert_eq!(gain, 91.29818322930544);
    }

    #[test]
    pub fn test_logarithm_13() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(TIME_FRAME * 3 / 4, TIME_FRAME, 1);
        assert_eq!(gain, 82.59636645861085);
    }

    #[test]
    pub fn test_logarithm_14() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(TIME_FRAME * 40 / 48, TIME_FRAME, 1);
        assert_eq!(gain, 77.50612995960805);
    }

    #[test]
    pub fn test_logarithm_15() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(TIME_FRAME * 44 / 48, TIME_FRAME, 1);
        assert_eq!(gain, 68.80431318891345);
    }

    #[test]
    pub fn test_logarithm_16() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(TIME_FRAME * 46 / 48, TIME_FRAME, 1);
        assert_eq!(gain, 60.10249641821888);
    }

    #[test]
    pub fn test_logarithm_17() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(TIME_FRAME * 47 / 48, TIME_FRAME, 1);
        assert_eq!(gain, 51.4006796475243);
    }

    #[test]
    pub fn test_logarithm_20() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(0, NEGATIVE_TIME_FRAME, 1);
        assert_eq!(gain, 100.0);
    }

    #[test]
    pub fn test_logarithm_21() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(NEGATIVE_TIME_FRAME / 4, NEGATIVE_TIME_FRAME, 1);
        assert_eq!(gain, 96.56335317912617);
    }

    #[test]
    pub fn test_logarithm_22() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(NEGATIVE_TIME_FRAME / 2, NEGATIVE_TIME_FRAME, 1);
        assert_eq!(gain, 91.71967153125861);
    }

    #[test]
    pub fn test_logarithm_23() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(NEGATIVE_TIME_FRAME * 54 / 72, NEGATIVE_TIME_FRAME, 1);
        assert_eq!(gain, 83.43934306251724);
    }

    #[test]
    pub fn test_logarithm_24() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(NEGATIVE_TIME_FRAME * 60 / 72, NEGATIVE_TIME_FRAME, 1);
        assert_eq!(gain, 78.5956614146497);
    }

    #[test]
    pub fn test_logarithm_25() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(NEGATIVE_TIME_FRAME * 64 / 72, NEGATIVE_TIME_FRAME, 1);
        assert_eq!(gain, 73.75197976678214);
    }

    #[test]
    pub fn test_logarithm_26() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(NEGATIVE_TIME_FRAME * 70 / 72, NEGATIVE_TIME_FRAME, 1);
        assert_eq!(gain, 57.19132282929937);
    }

    #[test]
    pub fn test_logarithm_27() {
        let (_x, _y, gain, _reverse_gain) = cutils::calc_log(NEGATIVE_TIME_FRAME * 71 / 72, NEGATIVE_TIME_FRAME, 1);
        assert_eq!(gain, 48.91099436055798);
    }
}