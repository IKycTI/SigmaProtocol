use num_bigint::{BigInt, BigUint};
use num_integer::Integer;
use num_traits::{One, Zero};

pub fn gcd_big(a: &BigUint, b: &BigUint) -> BigUint {
    if b == &BigUint::zero() {
        a.clone()
    } else {
        gcd_big(&b, &(a % b))
    }
}

pub fn mod_pow_big(base: &BigUint, exponent: &BigInt, modulus: &BigUint) -> Option<BigUint> {
    if modulus == &BigUint::zero() {
        return None;
    }

    if modulus == &BigUint::one() {
        return Some(BigUint::zero());
    }

    let result = if exponent < &BigInt::zero() {
        match modular_inverse_euclidean(&base, modulus) {
            Some(inv_base) => {
                mod_pow_positive_big(&inv_base, &(-exponent).to_biguint().unwrap(), modulus)
            }
            None => return None,
        }
    } else {
        mod_pow_positive_big(&base, &exponent.to_biguint().unwrap(), modulus)
    };

    Some(result)
}

fn mod_pow_positive_big(base: &BigUint, exponent: &BigUint, modulus: &BigUint) -> BigUint {
    if modulus.is_one() {
        return BigUint::zero();
    }

    let mut result = BigUint::one();
    let mut base = base % modulus;
    let mut exponent = exponent.clone();

    while !exponent.is_zero() {
        if exponent.is_odd() {
            result = (result * &base) % modulus;
        }
        base = (&base * &base) % modulus;
        exponent >>= 1;
    }

    result
}

fn extended_euclidean(a: &BigInt, b: &BigInt) -> (BigInt, BigInt, BigInt) {
    if a == &BigInt::zero() {
        return (b.clone(), BigInt::zero(), BigInt::one());
    }
    let (gcd, x1, y1) = extended_euclidean(&(b % a), a);
    let x = y1 - (b / a) * &x1;
    let y = x1;
    (gcd, x, y)
}

pub fn modular_inverse_euclidean(a: &BigUint, m: &BigUint) -> Option<BigUint> {
    let a = BigInt::from(a.clone());
    let m = BigInt::from(m.clone());
    if a == BigInt::zero() {
        return None;
    }
    if m == BigInt::one() {
        return None;
    }
    let (gcd, x, _) = extended_euclidean(&a, &m);
    if gcd != BigInt::one() {
        return None;
    }
    let mut result = x % &m;
    if result < BigInt::zero() {
        result += &m;
    }

    result.to_biguint()
}

#[cfg(test)]
mod tests {
    use num_traits::FromPrimitive;

    use super::*;

    ///////////////////////////////////
    ///          GCD               ///
    /////////////////////////////////
    #[test]
    fn test_gcd_small_numbers() {
        let a = BigUint::from(12u32);
        let b = BigUint::from(18u32);
        assert_eq!(gcd_big(&a, &b), BigUint::from(6u32));

        let a = BigUint::from(17u32);
        let b = BigUint::from(13u32);
        assert_eq!(gcd_big(&a, &b), BigUint::from(1u32));
    }

    #[test]
    fn test_gcd_with_zero() {
        let a = BigUint::from(0u32);
        let b = BigUint::from(15u32);
        assert_eq!(gcd_big(&a, &b), BigUint::from(15u32));

        let a = BigUint::from(42u32);
        let b = BigUint::zero();
        assert_eq!(gcd_big(&a, &b), BigUint::from(42u32));

        let a = BigUint::zero();
        let b = BigUint::zero();
        assert_eq!(gcd_big(&a, &b), BigUint::zero());
    }

    #[test]
    fn test_gcd_with_one() {
        let a = BigUint::one();
        let b = BigUint::from(12345u32);
        assert_eq!(gcd_big(&a, &b), BigUint::one());

        let a = BigUint::from(999u32);
        let b = BigUint::one();
        assert_eq!(gcd_big(&a, &b), BigUint::one());
    }

    #[test]
    fn test_gcd_equal_numbers() {
        let a = BigUint::from(42u32);
        let b = BigUint::from(42u32);
        assert_eq!(gcd_big(&a, &b), BigUint::from(42u32));

        let a = BigUint::from(1u32);
        let b = BigUint::from(1u32);
        assert_eq!(gcd_big(&a, &b), BigUint::one());
    }

    #[test]
    fn test_gcd_large_numbers() {
        let a = BigUint::from(123456789u32);
        let b = BigUint::from(987654321u32);
        let result = gcd_big(&a, &b);
        assert_eq!(result, BigUint::from(9u32));

        let a = BigUint::from(1_000_000_000u32);
        let b = BigUint::from(500_000_000u32);
        assert_eq!(gcd_big(&a, &b), BigUint::from(500_000_000u32));
    }

    #[test]
    fn test_gcd_very_large_numbers() {
        let a = BigUint::parse_bytes(b"123456789012345678901234567890", 10).unwrap();
        let b = BigUint::parse_bytes(b"987654321098765432109876543210", 10).unwrap();
        let result = gcd_big(&a, &b);
        let expected = BigUint::parse_bytes(b"9000000000900000000090", 10).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_gcd_prime_numbers() {
        let a = BigUint::from(17u32);
        let b = BigUint::from(19u32);
        assert_eq!(gcd_big(&a, &b), BigUint::one());

        let a = BigUint::from(101u32);
        let b = BigUint::from(103u32);
        assert_eq!(gcd_big(&a, &b), BigUint::one());
    }

    #[test]
    fn test_gcd_commutative() {
        let a = BigUint::from(56u32);
        let b = BigUint::from(98u32);
        assert_eq!(gcd_big(&a, &b), gcd_big(&b, &a));

        let a = BigUint::from(12345u32);
        let b = BigUint::from(67890u32);
        assert_eq!(gcd_big(&a, &b), gcd_big(&b, &a));
    }

    #[test]
    fn test_gcd_associative() {
        let a = BigUint::from(12u32);
        let b = BigUint::from(18u32);
        let c = BigUint::from(24u32);

        let left = gcd_big(&a, &gcd_big(&b, &c));
        let right = gcd_big(&gcd_big(&a, &b), &c);

        assert_eq!(left, right);
        assert_eq!(left, BigUint::from(6u32));
    }

    #[test]
    fn test_gcd_multiples() {
        let a = BigUint::from(15u32);
        let b = BigUint::from(45u32);
        assert_eq!(gcd_big(&a, &b), BigUint::from(15u32));

        let a = BigUint::from(7u32);
        let b = BigUint::from(49u32);
        assert_eq!(gcd_big(&a, &b), BigUint::from(7u32));
    }

    #[test]
    fn test_gcd_from_u64() {
        let a = BigUint::from_u64(18446744073709551615u64).unwrap();
        let b = BigUint::from_u64(1u64).unwrap();
        assert_eq!(gcd_big(&a, &b), BigUint::one());

        let a = BigUint::from_u64(2u64 * 3 * 5 * 7 * 11 * 13).unwrap();
        let b = BigUint::from_u64(3 * 5 * 7 * 17 * 19).unwrap();
        assert_eq!(gcd_big(&a, &b), BigUint::from_u64(3 * 5 * 7).unwrap());
    }

    //////////////////////////////////
    ///          MOD POW           ///
    /////////////////////////////////
    #[test]
    fn test_mod_pow_positive_exponent() {
        let base = BigUint::from(3u32);
        let exponent = BigInt::from(4i32);
        let modulus = BigUint::from(10u32);
        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert_eq!(result, BigUint::from(1u32));

        let base = BigUint::from(2u32);
        let exponent = BigInt::from(10i32);
        let modulus = BigUint::from(100u32);
        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert_eq!(result, BigUint::from(24u32));
    }

    #[test]
    fn test_mod_pow_zero_exponent() {
        let base = BigUint::from(5u32);
        let exponent = BigInt::zero();
        let modulus = BigUint::from(7u32);
        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert_eq!(result, BigUint::one());

        let base = BigUint::from(0u32);
        let exponent = BigInt::zero();
        let modulus = BigUint::from(13u32);
        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert_eq!(result, BigUint::one());
    }

    #[test]
    fn test_mod_pow_negative_exponent() {
        let base = BigUint::from(3u32);
        let exponent = BigInt::from(-1i32);
        let modulus = BigUint::from(11u32);
        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert_eq!(result, BigUint::from(4u32));

        let base = BigUint::from(2u32);
        let exponent = BigInt::from(-3i32);
        let modulus = BigUint::from(7u32);
        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert_eq!(result, BigUint::from(1u32));
    }

    #[test]
    fn test_mod_pow_modulus_one() {
        let base = BigUint::from(123u32);
        let exponent = BigInt::from(456i32);
        let modulus = BigUint::one();
        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert_eq!(result, BigUint::zero());
    }

    #[test]
    fn test_mod_pow_zero_base() {
        let base = BigUint::zero();
        let exponent = BigInt::from(5i32);
        let modulus = BigUint::from(10u32);
        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert_eq!(result, BigUint::zero());

        let base = BigUint::zero();
        let exponent = BigInt::from(-2i32);
        let modulus = BigUint::from(10u32);
        let result = mod_pow_big(&base, &exponent, &modulus);
        assert!(result.is_none());
    }

    #[test]
    fn test_mod_pow_large_numbers() {
        let base = BigUint::from(123456789u32);
        let exponent = BigInt::from(100i32);
        let modulus = BigUint::from(1000000007u32);
        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert!(result < modulus);

        let base = BigUint::from(2u32);
        let exponent = BigInt::from(1000i32);
        let modulus = BigUint::from(997u32);
        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert!(result < modulus);
    }

    #[test]
    fn test_mod_pow_very_large_numbers() {
        let base = BigUint::parse_bytes(b"123456789012345678901234567890", 10).unwrap();
        let exponent = BigInt::parse_bytes(b"1000", 10).unwrap();
        let modulus =
            BigUint::parse_bytes(b"10000000000000000000000000000000000000000", 10).unwrap();

        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert!(result < modulus);
    }

    #[test]
    fn test_mod_pow_edge_cases() {
        let base = BigUint::from(7u32);
        let exponent = BigInt::one();
        let modulus = BigUint::from(13u32);
        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert_eq!(result, BigUint::from(7u32));

        let base = BigUint::from(13u32);
        let exponent = BigInt::from(5i32);
        let modulus = BigUint::from(13u32);
        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert_eq!(result, BigUint::zero());

        let base = BigUint::from(20u32);
        let exponent = BigInt::from(3i32);
        let modulus = BigUint::from(13u32);
        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert_eq!(result, BigUint::from(5u32));
    }

    #[test]
    fn test_mod_pow_negative_base_handling() {
        let base = BigUint::from(3u32);
        let exponent = BigInt::from(-2i32);
        let modulus = BigUint::from(11u32);
        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert_eq!(result, BigUint::from(5u32));
    }

    #[test]
    fn test_mod_pow_no_modular_inverse() {
        let base = BigUint::from(2u32);
        let exponent = BigInt::from(-1i32);
        let modulus = BigUint::from(4u32);
        let result = mod_pow_big(&base, &exponent, &modulus);
        assert!(result.is_none());

        let base = BigUint::from(6u32);
        let exponent = BigInt::from(-1i32);
        let modulus = BigUint::from(9u32);
        let result = mod_pow_big(&base, &exponent, &modulus);
        assert!(result.is_none());
    }

    #[test]
    fn test_mod_pow_fermat_little_theorem() {
        let prime = BigUint::from(17u32);
        for a in 1..17 {
            if a % 17 != 0 {
                let base = BigUint::from(a as u32);
                let exponent = BigInt::from(16i32); // p-1
                let result = mod_pow_big(&base, &exponent, &prime).unwrap();
                assert_eq!(result, BigUint::one());
            }
        }
    }

    #[test]
    fn test_mod_pow_exponent_zero_modulus_one() {
        let base = BigUint::zero();
        let exponent = BigInt::zero();
        let modulus = BigUint::one();
        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert_eq!(result, BigUint::zero());
    }

    #[test]
    fn test_mod_pow_consistency() {
        let base = BigUint::from(5u32);
        let modulus = BigUint::from(13u32);

        let m = BigInt::from(3i32);
        let n = BigInt::from(4i32);

        let pow_m = mod_pow_big(&base, &m, &modulus).unwrap();
        let pow_n = mod_pow_big(&base, &n, &modulus).unwrap();
        let pow_m_n = mod_pow_big(&base, &(m + n), &modulus).unwrap();
        let product = (&pow_m * &pow_n) % &modulus;

        assert_eq!(pow_m_n, product);
    }

    #[test]
    fn test_mod_pow_negative_exponent_large() {
        let base = BigUint::from(7u32);
        let exponent = BigInt::from(-100i32);
        let modulus = BigUint::from(101u32);

        let result = mod_pow_big(&base, &exponent, &modulus).unwrap();
        assert!(result < modulus);
        assert!(result > BigUint::zero());
    }

    //////////////////////////////////
    ///    INVERSE EUCLIDIAN       ///
    /////////////////////////////////
    #[test]
    fn test_modular_inverse_basic() {
        let a = BigUint::from(3u32);
        let m = BigUint::from(11u32);
        let result = modular_inverse_euclidean(&a, &m).unwrap();
        assert_eq!(result, BigUint::from(4u32));

        let a = BigUint::from(7u32);
        let m = BigUint::from(13u32);
        let result = modular_inverse_euclidean(&a, &m).unwrap();
        assert_eq!(result, BigUint::from(2u32));
    }

    #[test]
    fn test_modular_inverse_no_solution() {
        let a = BigUint::from(2u32);
        let m = BigUint::from(4u32);
        let result = modular_inverse_euclidean(&a, &m);
        assert!(result.is_none());

        let a = BigUint::from(6u32);
        let m = BigUint::from(9u32);
        let result = modular_inverse_euclidean(&a, &m);
        assert!(result.is_none());

        let a = BigUint::from(10u32);
        let m = BigUint::from(15u32);
        let result = modular_inverse_euclidean(&a, &m);
        assert!(result.is_none());
    }

    #[test]
    fn test_modular_inverse_with_one() {
        let a = BigUint::one();
        let m = BigUint::from(7u32);
        let result = modular_inverse_euclidean(&a, &m).unwrap();
        assert_eq!(result, BigUint::one());

        let a = BigUint::one();
        let m = BigUint::from(100u32);
        let result = modular_inverse_euclidean(&a, &m).unwrap();
        assert_eq!(result, BigUint::one());
    }

    #[test]
    fn test_modular_inverse_self_inverse() {
        let a = BigUint::from(1u32);
        let m = BigUint::from(2u32);
        let result = modular_inverse_euclidean(&a, &m).unwrap();
        assert_eq!(result, BigUint::one());

        let a = BigUint::from(12u32);
        let m = BigUint::from(13u32);
        let result = modular_inverse_euclidean(&a, &m).unwrap();
        assert_eq!(result, BigUint::from(12u32));
    }

    #[test]
    fn test_modular_inverse_prime_modulus() {
        let prime = BigUint::from(17u32);
        for i in 1..17 {
            let a = BigUint::from(i as u32);
            let result = modular_inverse_euclidean(&a, &prime);
            assert!(result.is_some());

            let inv = result.unwrap();
            let product = (&a * &inv) % &prime;
            assert_eq!(product, BigUint::one());
        }
    }

    #[test]
    fn test_modular_inverse_composite_modulus() {
        let m = BigUint::from(15u32);
        let coprime_with_15 = vec![1, 2, 4, 7, 8, 11, 13, 14];
        for &i in &coprime_with_15 {
            let a = BigUint::from(i as u32);
            let result = modular_inverse_euclidean(&a, &m);
            assert!(result.is_some(), "Обратный должен существовать для {}", i);

            let inv = result.unwrap();
            let product = (&a * &inv) % &m;
            assert_eq!(product, BigUint::one());
        }

        let not_coprime_with_15 = vec![3, 5, 6, 9, 10, 12];
        for &i in &not_coprime_with_15 {
            let a = BigUint::from(i as u32);
            let result = modular_inverse_euclidean(&a, &m);
            assert!(
                result.is_none(),
                "Обратный не должен существовать для {}",
                i
            );
        }
    }

    #[test]
    fn test_modular_inverse_large_numbers() {
        let a = BigUint::from(123456789u32);
        let m = BigUint::from(1000000007u32);
        let result = modular_inverse_euclidean(&a, &m).unwrap();

        let product = (&a * &result) % &m;
        assert_eq!(product, BigUint::one());
        assert!(result < m);
    }

    #[test]
    fn test_modular_inverse_very_large_numbers() {
        let a = BigUint::parse_bytes(b"123456789012345678901234567890", 10).unwrap();
        let m = BigUint::parse_bytes(b"123456789012345678901234567891", 10).unwrap();
        let result = modular_inverse_euclidean(&a, &m).unwrap();

        let product = (&a * &result) % &m;
        assert_eq!(product, BigUint::one());
        assert!(result < m);
    }

    #[test]
    fn test_modular_inverse_edge_cases() {
        let m = BigUint::from(17u32);
        let a = &m - BigUint::one();
        let result = modular_inverse_euclidean(&a, &m);
        assert!(result.is_none());
        let a = BigUint::zero();
        let m = BigUint::from(7u32);
        let result = modular_inverse_euclidean(&a, &m);
        assert!(result.is_none());
    }

    #[test]
    fn test_modular_inverse_modulus_one() {
        let a = BigUint::from(0u32);
        let m = BigUint::one();
        let result = modular_inverse_euclidean(&a, &m);
        assert!(result.is_none());

        let a = BigUint::from(1u32);
        let m = BigUint::one();
        let result = modular_inverse_euclidean(&a, &m);
        assert!(result.is_none());
    }

    #[test]
    fn test_modular_inverse_modulus_two() {
        let m = BigUint::from(2u32);

        let a = BigUint::from(0u32);
        let result = modular_inverse_euclidean(&a, &m);
        assert!(result.is_none());

        let a = BigUint::from(1u32);
        let result = modular_inverse_euclidean(&a, &m).unwrap();
        assert_eq!(result, BigUint::one());
    }

    #[test]
    fn test_modular_inverse_consistency() {
        let a = BigUint::from(5u32);
        let m = BigUint::from(17u32);

        let inv1 = modular_inverse_euclidean(&a, &m).unwrap();
        let inv2 = modular_inverse_euclidean(&inv1, &m).unwrap();

        assert_eq!(inv2, a);
    }

    #[test]
    fn test_modular_inverse_product_rule() {
        let m = BigUint::from(23u32);

        let a = BigUint::from(3u32);
        let b = BigUint::from(5u32);

        let ab = (&a * &b) % &m;
        let inv_ab = modular_inverse_euclidean(&ab, &m).unwrap();

        let inv_a = modular_inverse_euclidean(&a, &m).unwrap();
        let inv_b = modular_inverse_euclidean(&b, &m).unwrap();
        let inv_product = (&inv_a * &inv_b) % &m;

        assert_eq!(inv_ab, inv_product);
    }

    #[test]
    fn test_modular_inverse_large_composite_modulus() {
        let a = BigUint::from(17u32);
        let m = BigUint::from(100u32);

        let result = modular_inverse_euclidean(&a, &m).unwrap();
        let product = (&a * &result) % &m;
        assert_eq!(product, BigUint::one());

        let a_not_coprime = BigUint::from(2u32);
        let result = modular_inverse_euclidean(&a_not_coprime, &m);
        assert!(result.is_none());
    }

    #[test]
    fn test_modular_inverse_random_cases() {
        let test_cases = vec![
            (7u32, 13u32, Some(2u32)),
            (9u32, 11u32, Some(5u32)),
            (4u32, 7u32, Some(2u32)),
            (8u32, 9u32, Some(8u32)),
            (3u32, 6u32, None),
            (10u32, 15u32, None),
        ];

        for (a, m, expected) in test_cases {
            let result = modular_inverse_euclidean(&BigUint::from(a), &BigUint::from(m));
            match expected {
                Some(inv) => {
                    assert!(result.is_some());
                    assert_eq!(result.unwrap(), BigUint::from(inv));
                }
                None => {
                    assert!(result.is_none());
                }
            }
        }
    }
}
