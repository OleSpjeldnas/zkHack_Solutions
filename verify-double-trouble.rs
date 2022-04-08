#![allow(unused, unreachable_code)]
use ark_ed_on_bls12_381::Fr;
use ark_ff::Field;
use double_trouble::data::puzzle_data;
use double_trouble::inner_product_argument::utils::challenge;
use double_trouble::verify;
use double_trouble::PUZZLE_DESCRIPTION;
use prompt::{puzzle, welcome};
use std::ops::Mul;

fn main() {
    welcome();
    puzzle(PUZZLE_DESCRIPTION);
    let (ck, [instance_and_proof_1, instance_and_proof_2]) = puzzle_data();
    let (instance1, proof1) = instance_and_proof_1;
    let (instance2, proof2) = instance_and_proof_2;
    assert!(verify(&ck, &instance1, &proof1));
    assert!(verify(&ck, &instance2, &proof2));

    let (a, comm_a_rand): (Vec<Fr>, Fr) = {
        let challenge1 = challenge(&ck, &instance1, &proof1.commitment);
let challenge2 = challenge(&ck, &instance2, &proof2.commitment);

let r1: Vec<Fr> = solve_for_r(
    proof1.response.s.clone(),
    proof2.response.s.clone(),
    challenge1,
    challenge2,
);

let a: Vec<Fr> = solve_for_a(proof1.response.s.clone(), challenge1, r1);

let comm_r_rand: Fr =
    solve_for_rho(proof1.response.u, proof2.response.u, challenge1, challenge2);

let comm_a_rand: Fr = solve_for_alpha(proof1.response.u, challenge1, comm_r_rand);

(a, comm_a_rand)
    };
    assert_eq!(
        ck.commit_with_explicit_randomness(&a, comm_a_rand),
        instance1.comm_a
    );
    assert_eq!(
        ck.commit_with_explicit_randomness(&a, comm_a_rand),
        instance2.comm_a
    );
}

fn solve_for_r(s1: Vec<Fr>, s2: Vec<Fr>, challenge1: Fr, challenge2: Fr) -> Vec<Fr> {
    let challenge_diff = challenge1 - challenge2.double();
    let challenge_diff_inv = challenge_diff.inverse().unwrap();

    let mut r = Vec::with_capacity(s1.capacity());
    for (s1_num, s2_num) in s1.iter().zip(s2.iter()) {
        let s_diff_i = *s1_num - *s2_num;
        let r_i = s_diff_i * challenge_diff_inv;
        r.push(r_i);
    }

    r
}

fn solve_for_a(s1: Vec<Fr>, challenge1: Fr, r: Vec<Fr>) -> Vec<Fr> {
    let mut a = Vec::with_capacity(s1.capacity());

    for (s1_num, r_num) in s1.iter().zip(r.iter()) {
        let a_i = *s1_num - challenge1 * *r_num;
        a.push(a_i);
    }

    a
}

fn solve_for_rho(u1: Fr, u2: Fr, challenge1: Fr, challenge2: Fr) -> Fr {
    let challenge_diff_inv = (challenge1 - challenge2.double()).inverse().unwrap();
    (u1 - u2) * challenge_diff_inv
}

fn solve_for_alpha(u1: Fr, challenge1: Fr, rho1: Fr) -> Fr {
    u1 - challenge1 * rho1
}

