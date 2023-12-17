mod data;

#[cfg(test)]
mod tests {
    use i_overlay::bool::fill_rule::FillRule;
    use i_triangle::triangulate::Triangulate;
    use crate::data::triangulation::Test;

    fn execute(index: usize) {
        let test = Test::load(index);
        let polygons = test.shape.into_convex_polygons(Some(FillRule::EvenOdd));
        assert_eq!(polygons.is_empty(), false);
        assert_eq!(test.polygons, polygons);
    }

    #[test]
    fn test_0() {
        execute(0);
    }

    #[test]
    fn test_1() {
        execute(1);
    }

    #[test]
    fn test_2() {
        execute(2);
    }

    #[test]
    fn test_3() {
        execute(3);
    }

    #[test]
    fn test_4() {
        execute(4);
    }

    #[test]
    fn test_5() {
        execute(5);
    }

    #[test]
    fn test_6() {
        execute(6);
    }

    #[test]
    fn test_7() {
        execute(7);
    }

    #[test]
    fn test_8() {
        execute(8);
    }

    #[test]
    fn test_9() {
        execute(9);
    }

    #[test]
    fn test_10() {
        execute(10);
    }

    #[test]
    fn test_11() {
        execute(11);
    }

    #[test]
    fn test_12() {
        execute(12);
    }

    #[test]
    fn test_13() {
        execute(13);
    }

    #[test]
    fn test_14() {
        execute(14);
    }

    #[test]
    fn test_15() {
        execute(15);
    }

    #[test]
    fn test_16() {
        execute(16);
    }

    #[test]
    fn test_17() {
        execute(17);
    }

    #[test]
    fn test_18() {
        execute(18);
    }

    #[test]
    fn test_19() {
        execute(19);
    }

    #[test]
    fn test_20() {
        execute(20);
    }

    #[test]
    fn test_21() {
        execute(21);
    }

    #[test]
    fn test_22() {
        execute(22);
    }

    #[test]
    fn test_23() {
        execute(23);
    }

    #[test]
    fn test_24() {
        execute(24);
    }

    #[test]
    fn test_25() {
        execute(25);
    }

    #[test]
    fn test_26() {
        execute(26);
    }

    #[test]
    fn test_27() {
        execute(27);
    }

    #[test]
    fn test_28() {
        execute(28);
    }

    #[test]
    fn test_29() {
        execute(29);
    }

    #[test]
    fn test_30() {
        execute(30);
    }

    #[test]
    fn test_31() {
        execute(31);
    }

    #[test]
    fn test_32() {
        execute(32);
    }

    #[test]
    fn test_33() {
        execute(33);
    }

    #[test]
    fn test_34() {
        execute(34);
    }

    #[test]
    fn test_35() {
        execute(35);
    }

    #[test]
    fn test_36() {
        execute(36);
    }

    #[test]
    fn test_37() {
        execute(37);
    }

    #[test]
    fn test_38() {
        execute(38);
    }

    #[test]
    fn test_39() {
        execute(39);
    }

    #[test]
    fn test_40() {
        execute(40);
    }

    #[test]
    fn test_41() {
        execute(41);
    }

    #[test]
    fn test_42() {
        execute(42);
    }

    #[test]
    fn test_43() {
        execute(43);
    }

    #[test]
    fn test_44() {
        execute(44);
    }

    #[test]
    fn test_45() {
        execute(45);
    }

    #[test]
    fn test_46() {
        execute(46);
    }

    #[test]
    fn test_47() {
        execute(47);
    }

    #[test]
    fn test_48() {
        execute(48);
    }

    #[test]
    fn test_49() {
        execute(49);
    }

    #[test]
    fn test_50() {
        execute(50);
    }

    #[test]
    fn test_51() {
        execute(51);
    }

    #[test]
    fn test_52() {
        execute(52);
    }

    #[test]
    fn test_53() {
        execute(53);
    }

    #[test]
    fn test_54() {
        execute(54);
    }

    #[test]
    fn test_55() {
        execute(55);
    }

    #[test]
    fn test_56() {
        execute(56);
    }

    #[test]
    fn test_57() {
        execute(57);
    }

    #[test]
    fn test_58() {
        execute(58);
    }

    #[test]
    fn test_59() {
        execute(59);
    }

    #[test]
    fn test_60() {
        execute(60);
    }

    #[test]
    fn test_61() {
        execute(61);
    }

    #[test]
    fn test_62() {
        execute(62);
    }

    #[test]
    fn test_63() {
        execute(63);
    }

    #[test]
    fn test_64() {
        execute(64);
    }

    #[test]
    fn test_65() {
        execute(65);
    }

    #[test]
    fn test_66() {
        execute(66);
    }

    #[test]
    fn test_67() {
        execute(67);
    }

    #[test]
    fn test_68() {
        execute(68);
    }

    #[test]
    fn test_69() {
        execute(69);
    }

    #[test]
    fn test_70() {
        execute(70);
    }

}