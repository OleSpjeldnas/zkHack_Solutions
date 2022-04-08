#![allow(unused, unreachable_code, dead_code)]

use ark_bls12_381::{Fr, G1Affine};
use ark_ff::*;
use ark_poly::{
    univariate::DensePolynomial, EvaluationDomain, GeneralEvaluationDomain, Polynomial,
    UVPolynomial,
};
use ark_serialize::CanonicalDeserialize;
use hidden_in_plain_sight::{generate::kzg_commit, PUZZLE_DESCRIPTION};
use prompt::{puzzle, welcome};

fn read_cha_from_file() -> (Vec<G1Affine>, Vec<Vec<Fr>>, Fr, Fr, G1Affine, Fr, Fr) {
    use std::fs::File;
    use std::io::prelude::*;

    let mut file = File::open("challenge_data").unwrap();
    let mut bytes: Vec<u8> = vec![];
    file.read_to_end(&mut bytes).unwrap();

    let setup_bytes: Vec<u8> = bytes[0..98312].to_vec();
    let accts_bytes: Vec<u8> = bytes[98312..1130320].to_vec();
    let cha_1_bytes: Vec<u8> = bytes[1130320..1130352].to_vec();
    let cha_2_bytes: Vec<u8> = bytes[1130352..1130384].to_vec();
    let commt_bytes: Vec<u8> = bytes[1130384..1130480].to_vec();
    let opn_1_bytes: Vec<u8> = bytes[1130480..1130512].to_vec();
    let opn_2_bytes: Vec<u8> = bytes[1130512..1130544].to_vec();

    let setup = Vec::<G1Affine>::deserialize_unchecked(&setup_bytes[..]).unwrap();
    let accts = Vec::<Vec<Fr>>::deserialize_unchecked(&accts_bytes[..]).unwrap();
    let cha_1 = Fr::deserialize_unchecked(&cha_1_bytes[..]).unwrap();
    let cha_2 = Fr::deserialize_unchecked(&cha_2_bytes[..]).unwrap();
    let commt = G1Affine::deserialize_unchecked(&commt_bytes[..]).unwrap();
    let opn_1 = Fr::deserialize_unchecked(&opn_1_bytes[..]).unwrap();
    let opn_2 = Fr::deserialize_unchecked(&opn_2_bytes[..]).unwrap();

    (setup, accts, cha_1, cha_2, commt, opn_1, opn_2)
}

fn main() {
    welcome();
    puzzle(PUZZLE_DESCRIPTION);

    let (setup, accts, cha_1, cha_2, commt, opn_1, opn_2) = read_cha_from_file();

    // Replace with the solution polynomial, derived from the account!
    let solution_blinded_acct = DensePolynomial::from_coefficients_vec(find_poly(&cha_1, &cha_2, &opn_1, &opn_2, &accts, &setup, &commt).unwrap());

    let solution_commitment = kzg_commit(&solution_blinded_acct, &setup);
    assert_eq!(solution_commitment, commt);
}

fn find_blindings(cha_1 : &Fr, cha_2 : &Fr, opening_1 : &Fr, opening_2 : &Fr, poly : &Vec<Fr>)  ->
(Fr, Fr) {
    let domain: GeneralEvaluationDomain<Fr> =
        GeneralEvaluationDomain::new(1000usize + 2).unwrap();
    let p = DensePolynomial::from_coefficients_vec(domain.ifft(poly));
    let p_1 : Fr = p.evaluate(cha_1);
    let p_2 : Fr = p.evaluate(cha_2);
    const N: u64 = 1024u64;
    let van_1 = opening_1.pow(&[N])- Fr::from(1 as i32);;
    let van_2 = opening_2.pow(&[N])- Fr::from(1 as i32);;

    let b1 = (*cha_1 + *cha_2).inverse().unwrap()*((*cha_2-p_2)/van_2 + (*cha_1-p_1)/van_1);
    let b0 = (*cha_1-p_1)/van_1 -b1*opening_1;

    (b0, b1)
}

fn compute_commitment(setup : &Vec<G1Affine>, b0 : Fr, b1 : Fr, acct : &Vec<Fr>) -> G1Affine {
    let domain: GeneralEvaluationDomain<Fr> =
        GeneralEvaluationDomain::new(1000usize + 2).unwrap();
    let target_acct_poly = DensePolynomial::from_coefficients_vec(domain.ifft(acct));
    let blinding_poly = DensePolynomial::from_coefficients_vec(vec![b0, b1]);
    let blinded_acct_poly = target_acct_poly + blinding_poly.mul_by_vanishing_poly(domain);

    let commitment: G1Affine = kzg_commit(&blinded_acct_poly, &setup);

    commitment
}

fn check_commitment_match(commitment : &G1Affine, ground_truth : &G1Affine) -> bool {
    commitment == ground_truth
}

fn find_poly(cha_1 : &Fr, cha_2 : &Fr, opening_1 : &Fr, opening_2 : &Fr, accts : &Vec<Vec<Fr>>, setup : &Vec<G1Affine>, commitment : &G1Affine) 
-> Option<Vec<Fr>>{
    for acct in accts.iter() {
        let (b0, b1) = find_blindings(cha_1, cha_2, opening_1, opening_2, &acct);
        let c = compute_commitment(setup, b0, b1, acct);
        if (check_commitment_match(&c, &commitment)) {
            return Some(acct.to_vec())
        }
        
    }
    None
}