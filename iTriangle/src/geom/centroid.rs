use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::product_uint::UIntProduct;
use i_overlay::i_float::int::number::signed_product::SignedProduct;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::point::IntPoint;

// Area centroid rounded to the integer grid relative to origin.
pub(crate) fn polygon_centroid<I: IntNumber>(
    polygon: &[IntPoint<I>],
    origin: IntPoint<I>,
) -> Option<IntPoint<I>> {
    if polygon.len() < 3 {
        return None;
    }

    let mut area_two = I::Wide::ZERO;
    let mut x_numerator = SignedProduct::<I::Wide>::multiply(I::Wide::ZERO, I::Wide::ZERO);
    let mut y_numerator = SignedProduct::<I::Wide>::multiply(I::Wide::ZERO, I::Wide::ZERO);

    for index in 0..polygon.len() {
        let a = polygon[index] - origin;
        let b = polygon[(index + 1) % polygon.len()] - origin;
        let cross = a.cross_product(b);

        area_two = area_two + cross;
        x_numerator = x_numerator.checked_add(SignedProduct::multiply(a.x + b.x, cross))?;
        y_numerator = y_numerator.checked_add(SignedProduct::multiply(a.y + b.y, cross))?;
    }

    if area_two <= I::Wide::ZERO {
        return None;
    }

    let three = I::WideUInt::from_u64(3);
    let area = area_two.to_uint();
    if area > (I::WideUInt::LAST_BIT - I::WideUInt::ONE) / three {
        return None;
    }
    let divisor = area * three;

    let dx = divide_signed_product(x_numerator, divisor)?;
    let dy = divide_signed_product(y_numerator, divisor)?;

    Some(IntPoint::new(
        I::from_wide(origin.x.to_wide() + dx),
        I::from_wide(origin.y.to_wide() + dy),
    ))
}

fn divide_signed_product<I: WideIntNumber>(value: SignedProduct<I>, divisor: I::UInt) -> Option<I> {
    debug_assert!(divisor > I::UInt::ZERO);
    debug_assert!(divisor < I::UInt::LAST_BIT);

    let magnitude = value.magnitude().divide_with_rounding(divisor);
    if magnitude >= I::UInt::LAST_BIT {
        return None;
    }

    let result = I::from_uint(magnitude);
    Some(if value.is_negative() { -result } else { result })
}
