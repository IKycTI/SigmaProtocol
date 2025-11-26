use num_bigint::{BigInt, BigUint, RandBigInt, ToBigInt, ToBigUint};
use num_traits::{FromPrimitive, One, Zero};

use crate::math;

const RANDOM_SIZE: u64 = 64;

//Генерация случайного простого числа
pub async fn gen_random_prime() -> BigUint {
    let mut rng = rand::thread_rng();
    let mut res = rng.gen_biguint(RANDOM_SIZE);
    if &res % BigUint::from_u8(2).unwrap() == BigUint::zero() {
        res += BigUint::one();
    }

    while !is_prime_miller_rabin(&res, 8) {
        res += BigUint::from_u8(2).unwrap();
    }
    res.to_biguint().unwrap()
}

pub async fn random_biguint_mod(module: &BigUint) -> BigUint {
    let mut rng = rand::thread_rng();
    rng.gen_biguint(RANDOM_SIZE) % module
}

// pub async fn

fn is_prime_miller_rabin(n: &BigUint, k: u8) -> bool {
    if n <= &BigUint::one() {
        return false;
    }
    if n == &BigUint::from_u8(2).unwrap() || n == &BigUint::from_u8(3).unwrap() {
        return true;
    }
    if n % BigUint::from_u8(2).unwrap() == BigUint::zero() {
        return false;
    }
    let mut t: BigInt = (n - BigUint::one()).to_bigint().unwrap();
    let mut s = 0;
    while &t % 2 == BigInt::zero() {
        t = t / 2;
        s += 1;
    }
    'A: for _ in 0..k {
        let mut rng = rand::thread_rng();
        let a = rng.gen_biguint_range(
            &BigUint::from_u8(2).unwrap(),
            &(n - BigUint::from_u8(2).unwrap()),
        );
        let mut x = match math::mod_pow_big(&a, &t, n) {
            Some(x) => x,
            None => {
                eprintln!("Error in mod_pow");
                continue 'A;
            }
        };
        if x == BigUint::one() || x == n - BigUint::one() {
            continue 'A;
        }
        for _ in 0..s - 1 {
            x = match math::mod_pow_big(&x, &BigInt::from_i8(2).unwrap(), n) {
                Some(x) => x,
                None => {
                    eprintln!("Error in mod_pow");
                    continue 'A;
                }
            };
            if x == BigUint::one() {
                return false;
            }
            if x == n - BigUint::one() {
                continue 'A;
            }
        }
        return false;
    }
    true
}
